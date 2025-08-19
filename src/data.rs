use crate::args::Args;
use heimdall::{status::Statuses, storage::Storage};

pub struct Data {
    pub args: Args,
    pub statuses: Statuses,
    pub storage: Storage,
}

impl Data {
    pub fn new(args: Args, statuses: Statuses, storage: Storage) -> Self {
        Self {
            args,
            statuses,
            storage,
        }
    }
}
