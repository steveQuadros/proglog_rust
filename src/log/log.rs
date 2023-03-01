use serde::{Deserialize, Serialize};
use std::{
    fmt, str,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    #[serde(with = "serde_bytes")]
    pub message: Vec<u8>,
    #[serde(default)]
    pub offset: u64,
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", str::from_utf8(&self.message).unwrap())
    }
}

impl Record {
    pub fn new(message: Vec<u8>) -> Self {
        Record { message, offset: 0 }
    }
}

#[derive(Debug)]
pub struct Log {
    records: Arc<Mutex<Vec<Record>>>,
}

impl Log {
    pub fn new() -> Self {
        Log {
            records: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn append(&self, mut record: Record) -> u64 {
        let mut records = self.records.lock().unwrap();
        let offset = records.len() as u64;
        record.offset = offset;
        records.push(record);
        offset
    }

    pub fn read(&self, offset: u64) -> Record {
        let records = self.records.lock().unwrap();
        records[offset as usize].clone()
    }

    pub fn size(&self) -> usize {
        self.records.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_log() {
        let log = Log::new();
        assert_eq!(log.size(), 0);
    }

    #[test]
    fn append_and_read() {
        let log = Log::new();
        let msg: Vec<u8> = "foo".into();
        let msg_val = msg.clone();
        log.append(Record::new(msg));
        assert_eq!(log.size(), 1);

        let record = log.read(0);
        assert_eq!(record.message, msg_val);
    }
}
