use std::{sync::{Arc, Mutex}, fmt, str};

#[derive(Debug, Clone)]
pub struct Record<'a> {
    pub message: &'a [u8],
    pub offset: u64,
}

impl<'a> fmt::Display for Record<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", str::from_utf8(&self.message).unwrap())
    }
}

impl<'a> Record<'a> {
    pub fn new(message: &'a [u8]) -> Self {
        Record {message, offset: 0}
    }
}

#[derive(Debug)]
pub struct Log<'a> {
    records: Arc<Mutex<Vec<Record<'a>>>>,
}

impl<'a> Log<'a> {
    pub fn new() -> Self {
        Log {
            records: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn append(&self, mut record: Record<'a>) -> u64 {
        let mut records = self.records.lock().unwrap();
        let offset = records.len() as u64;
        record.offset = offset;
        records.push(record);
        offset
    }

    pub fn read(&self, offset: u64) -> Record  {
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
        let msg = "foo".as_bytes();
        log.append(Record::new(msg));
        assert_eq!(log.size(), 1);

        let record = log.read(0);
        assert_eq!(record.message, msg);
    }
}