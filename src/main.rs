mod log;
mod server;

use crate::log::log::Log;
use crate::server::Result as ServerResult;
use std::sync::Arc;

#[tokio::main]
async fn main() -> ServerResult<()> {
    let log = Arc::new(Log::new());
    server::start(log).await
}
