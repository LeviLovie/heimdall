use crate::prelude::RsLog;

pub mod prelude {
    pub use super::Storage;
}

pub struct Storage {
    logs: Vec<RsLog>,
}

impl Storage {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn add_log(&mut self, log: RsLog) {
        self.logs.push(log);
    }

    pub fn get_logs(&self) -> &[RsLog] {
        &self.logs
    }
}
