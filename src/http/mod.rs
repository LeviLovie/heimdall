use anyhow::Result;
use heimdall::status::ThreadType;
use std::sync::{Arc, Mutex};

use crate::data::Data;

pub fn receive(data: Arc<Mutex<Data>>, _port: u16) -> Result<()> {
    let print_info = !data.lock().unwrap().args.tui;

    loop {
        let must_terminate = {
            let data_lock = data.lock().unwrap();
            data_lock.statuses.must_terminate(ThreadType::HTTP)
        };
        if must_terminate {
            if print_info {
                println!("Terminating HTTP listener thread");
            }
            break;
        }
    }

    Ok(())
}
