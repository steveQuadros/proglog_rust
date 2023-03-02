use bytes::Buf;
use std::sync::Arc;
use url::Url;
// use futures_util::{stream, StreamExt};
use crate::log::log::{Log, Record};
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, GenericError>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

#[derive(Deserialize)]
struct CreateRecordReq {
    message: String,
}

#[derive(Serialize)]
struct CreateRecordRes {
    message: String,
    offset: u64,
}

impl CreateRecordRes {
    fn new(r: &Record) -> Self {
        CreateRecordRes {
            message: String::from_utf8(r.message.clone()).unwrap(),
            offset: r.offset,
        }
    }
}

pub async fn start(log: Arc<Log>) -> Result<()> {
    pretty_env_logger::init();
    let addr = "127.0.0.1:1337".parse().unwrap();
    let new_service = make_service_fn(move |_| {
        let log = log.clone();
        async move { Ok::<_, GenericError>(service_fn(move |req| response_examples(log.clone(), req))) }
    });

    let server = hyper::Server::bind(&addr).serve(new_service);
    // And now add a graceful shutdown signal...
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    println!("Listening on http://{}", addr);
    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

async fn api_post_response(log: Arc<Log>, req: Request<Body>) -> Result<Response<Body>> {
    // Aggregate the body...
    let whole_body = hyper::body::aggregate(req).await?;
    let data: Record = serde_json::from_reader(whole_body.reader())?;
    let mut res = CreateRecordRes::new(&data);
    res.offset = log.append(data);
    let json = serde_json::to_string(&res)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;
    Ok(response)
}

async fn api_get_response(log: Arc<Log>, req: Request<Body>) -> Result<Response<Body>> {
    let req_uri = req.uri().to_string();
    println!("{}", req_uri);
    let offset = req_uri.split('/').nth(2).unwrap();
    let data = log.read(offset.parse::<u64>().unwrap());
    let json = CreateRecordRes::new(&data);
    let res = match serde_json::to_string(&json) {
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

async fn response_examples(log: Arc<Log>, req: Request<Body>) -> Result<Response<Body>> {
    /*
     * Extract and normalize the segments from the URI path.
     */
    let path = req.uri().path().to_owned();
    let mut segments = Vec::new();

    for s in path.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }

    // Pass th segments to the routing handler.
    route(req, &segments, log).await
}

async fn route(req: Request<Body>, segments: &[&str], log: Arc<Log>) -> Result<Response<Body>> {
    match (req.method(), segments) {
        (&Method::POST, ["records"]) => api_post_response(log, req).await,
        // match on /records/1 and pass that offset in as int
        (&Method::GET, ["records", offset]) => api_get_response(log, req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOTFOUND.into())
                .unwrap())
        }
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
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
        let log = Arc::new(Log::new());
        let res = api_post_response(log, req).await;
        let body_bytes = hyper::body::to_bytes(res.unwrap().into_body())
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(body_bytes.to_vec()).unwrap(),
            "{\"message\":\"foobar\",\"offset\":0}"
        );
    }

    #[tokio::test]
    async fn get_record() {
        let req = Request::builder()
            .uri("/records/1")
            .body(Body::default())
            .unwrap();
        let log = Arc::new(Log::new());
        for _ in 0..3 {
            log.append(Record::new(b"foobar".to_vec()));
        }
        assert_eq!(log.size(), 3);

        let res = api_get_response(log, req).await;
        let body_bytes = hyper::body::to_bytes(res.unwrap().into_body())
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(body_bytes.to_vec()).unwrap(),
            "{\"message\":\"foobar\",\"offset\":1}"
        );
    }
}
