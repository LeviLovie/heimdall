use anyhow::{Context, Result};
use nng::{
    options::{Options, RemAddr},
    {Pipe, Protocol, Socket},
};
use std::sync::{Arc, Mutex};

use crate::data::Data;
use heimdall::{prelude::*, schemas::log::log::Log};

pub fn receive(data: Arc<Mutex<Data>>, port: u16) -> Result<()> {
    let bind = format!("tcp://{}:{}", data.lock().unwrap().args.address, port);
    let print_info = !data.lock().unwrap().args.tui;

    let mut socket = Socket::new(Protocol::Pull0).context("Failed to create a new socket")?;
    socket
        .listen(&bind)
        .context("Failed to bind socket to address")?;

    if print_info {
        println!("Listening for messages on {bind}");
    }

    loop {
        let must_terminate = {
            let data_lock = data.lock().unwrap();
            data_lock.statuses.must_terminate(ThreadType::NNG)
        };
        if must_terminate {
            if print_info {
                println!("Terminating NNG listener thread");
            }
            break;
        }

        match listen(&mut socket) {
            Err(e) => println!("Error: {:?}", e.context("Failed to recive message")),
            Ok(log) => {
                if let Some(log) = log {
                    if print_info {
                        println!("{log}");
                    }
                    data.lock()
                        .unwrap()
                        .storage
                        .add_log(log)
                        .context("Failed to add log to storage")?;
                }
            }
        };
    }

    Ok(())
}

fn listen(socket: &mut Socket) -> Result<Option<RsLog>> {
    match socket.try_recv() {
        Ok(mut msg) => {
            let pipe: Pipe = msg.pipe().context("Message missing pipe")?;
            let ip = pipe
                .get_opt::<RemAddr>()
                .context("Failed to get remote address")?
                .to_string();
            let buf: Vec<u8> = msg.as_slice().to_vec();
            let log =
                flatbuffers::root::<Log>(&buf).context("Failed to deserialize log message")?;
            Ok(Some(RsLog::from(log, ip)))
        }
        Err(nng::Error::TryAgain) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
