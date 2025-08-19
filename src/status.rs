use std::collections::HashMap;

pub mod prelude {
    pub use super::{Statuses, ThreadStatus, ThreadType};
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ThreadType {
    TUI,
    NNG,
    HTTP,
}

#[derive(Debug, Clone)]
pub enum ThreadStatus {
    Running,
    Stopped,
    Failed(String),
}

pub struct Statuses {
    statuses: HashMap<ThreadType, ThreadStatus>,
}

impl Statuses {
    pub fn new() -> Self {
        Self {
            statuses: HashMap::new(),
        }
    }

    pub fn set(&mut self, thread_type: ThreadType, status: ThreadStatus) {
        self.statuses.insert(thread_type, status);
    }

    pub fn get(&self, thread_type: ThreadType) -> Option<&ThreadStatus> {
        self.statuses.get(&thread_type)
    }

    pub fn get_all(&self) -> Vec<(ThreadType, ThreadStatus)> {
        self.statuses
            .iter()
            .map(|(thread_type, status)| (thread_type.clone(), status.clone()))
            .collect()
    }
}
