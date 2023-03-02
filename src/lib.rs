pub mod proglog_rust {
    pub mod records {
        include!(concat!(env!("OUT_DIR"), "/proglog_rust.log.rs"));
    }
}

use proglog_rust::records;
