use anyhow::{Result, bail};
use std::sync::{Arc, Mutex};

use crate::args::Args;
use heimdall::storage::Storage;

pub fn receive(_args: Args, _port: u16, _storate: Arc<Mutex<Storage>>) -> Result<()> {
    bail!("HTTP server is not implemented yet. Please use NNG server instead.");
}
