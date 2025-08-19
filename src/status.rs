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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ThreadStatus {
    Running,
    Terminating,
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

    pub fn must_terminate(&self, thread_type: ThreadType) -> bool {
        matches!(
            self.statuses.get(&thread_type),
            Some(ThreadStatus::Terminating)
        )
    }

    pub fn terminate_all(&mut self) {
        for status in self.statuses.values_mut() {
            if *status == ThreadStatus::Running {
                *status = ThreadStatus::Terminating;
            }
        }
    }

    pub fn all_stopped_except(&self, thread_type: ThreadType) -> bool {
        let res = self.statuses.iter().all(|(t, status)| {
            *t == thread_type
                || *status == ThreadStatus::Stopped
                || matches!(*status, ThreadStatus::Failed(_))
        });
        res
    }
}
