use bytes::Buf;
// use futures_util::{stream, StreamExt};
use crate::log::log::Record;
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

pub async fn start() -> Result<()> {
    pretty_env_logger::init();

    let addr = "127.0.0.1:1337".parse().unwrap();

    // Share a `Client` with all `Service`s
    let client = Client::new();

    let new_service = make_service_fn(move |_| {
        // Move a clone of `client` into the `service_fn`.
        let client = client.clone();
        async {
            Ok::<_, GenericError>(service_fn(move |req| {
                // Clone again to ensure that client outlives this closure.
                response_examples(req, client.to_owned())
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

#[derive(Deserialize)]
struct CreateRecordReq {
    message: String,
}

#[derive(Serialize)]
struct CreateRecordRes {
    message: String,
    offset: u64,
}

async fn api_post_response(req: Request<Body>) -> Result<Response<Body>> {
    // Aggregate the body...
    let whole_body = hyper::body::aggregate(req).await?;
    let data: Record = serde_json::from_reader(whole_body.reader())?;

    let res = CreateRecordRes {
        message: String::from_utf8(data.message).unwrap(),
        offset: data.offset,
    };
    let json = serde_json::to_string(&res)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;
    Ok(response)
}

async fn api_get_response() -> Result<Response<Body>> {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(INTERNAL_SERVER_ERROR.into())
            .unwrap(),
    };
    Ok(res)
}

async fn response_examples(
    req: Request<Body>,
    client: Client<HttpConnector>,
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/json_api") => api_post_response(req).await,
        (&Method::GET, "/json_api") => api_get_response().await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOTFOUND.into())
                .unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_stuff() {
        let data = r#"{"message": "foobar"}"#;
        let d: serde_json::Value = serde_json::from_str(data).unwrap();
        let r = Record::new(d["message"].as_str().unwrap().as_bytes().to_vec());
        assert_eq!(String::from_utf8(r.message).unwrap(), "foobar");
    }

    #[test]
    fn json_into_record() {
        let data = r#"{"message": "foo"}"#;
        let r: Record = serde_json::from_str(data).unwrap();
        assert_eq!(String::from_utf8(r.message).unwrap(), "foo");
    }

    #[tokio::test]
    async fn create_record() {
        let data = r#"{"message": "foobar"}"#;
        let req = Request::builder().body(Body::from(data)).unwrap();
        let res = api_post_response(req).await;
        let body_bytes = hyper::body::to_bytes(res.unwrap().into_body())
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(body_bytes.to_vec()).unwrap(),
            "{\"message\":\"foobar\",\"offset\":0}"
        );
    }
}
