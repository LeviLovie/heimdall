use crate::args::ServerArgs;
use heimdall::{status::Statuses, storage::Storage};

pub struct Data {
    pub args: ServerArgs,
    pub statuses: Statuses,
    pub storage: Storage,
}

impl Data {
    pub fn new(args: ServerArgs, statuses: Statuses, storage: Storage) -> Self {
        Self {
            args,
            statuses,
            storage,
        }
    }
}
