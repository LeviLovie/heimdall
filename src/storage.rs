use crate::prelude::RsLog;

pub mod prelude {
    pub use super::Storage;
}

pub struct Storage {
    logs: Vec<RsLog>,
    pub updated: bool,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            updated: false,
        }
    }

    pub fn add_log(&mut self, log: RsLog) {
        self.logs.push(log);
        self.updated = true;
    }

    pub fn get_logs(&self) -> &[RsLog] {
        &self.logs
    }

    pub fn was_updated(&self) -> bool {
        self.updated
    }
}
