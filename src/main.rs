mod log;

use crate::log::log::{Log, Record};

fn main() {
    let log = Log::new();
    dbg!(&log);
    println!("starting log size: {}", log.size());

    let items = ["foo", "bar", "baz", "quo"];
    for item in items {
        let r = Record::new(item.as_bytes());
        log.append(r);
    }

    println!("log size: {}", log.size());

    for n in 0..items.len() {
        let msg = log.read(n as u64);
        println!("msg: {}", msg);
    }
}
