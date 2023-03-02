mod log;
mod server;

use crate::log::log::Log;
use crate::server::Result as ServerResult;
use std::sync::Arc;

pub mod proglog_rust {
    pub mod records {
        // include!(concat!(env!("OUT_DIR"), "/proglog_rust.records.rs"));
    }
}

#[tokio::main]
async fn main() -> ServerResult<()> {
    let log = Arc::new(Log::new());
    server::start(log).await
}
