mod args;
mod tui;

use anyhow::{Context, Result};
use nng::options::{Options, RemAddr};
use nng::{Pipe, Protocol, Socket};
use std::sync::{Arc, Mutex};

use heimdall::prelude::*;
use heimdall::schemas::log::log::Log;

pub fn deserialize_log(buf: &[u8]) -> Result<Log> {
    flatbuffers::root::<Log>(buf).context("Failed to deserialize Log")
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = args::parse();

    let storage: Arc<Mutex<Storage>> = Arc::new(Mutex::new(Storage::new()));

    let storage_clone = storage.clone();
    let args_clone = args.clone();
    std::thread::spawn(move || {
        receive(args_clone, storage_clone).context("Failed to start a receiving server")
    });

    if args.tui {
        tui::start(storage).context("Failed to run TUI")?;
    } else {
        #[allow(clippy::empty_loop)]
        loop {}
    }

    Ok(())
}

fn receive(args: args::Args, storate: Arc<Mutex<Storage>>) -> Result<()> {
    let bind = format!("tcp://{}:{}", args.address, args.port);

    let mut socket = Socket::new(Protocol::Pull0).context("Failed to create a new socket")?;
    socket
        .listen(&bind)
        .context("Failed to bind socket to address")?;

    if !args.tui {
        println!("Listening for messages on {bind}");
    }

    loop {
        match listen(&mut socket) {
            Err(e) => println!("Error: {:?}", e.context("Failed to recive message")),
            Ok(log) => {
                if !args.tui {
                    println!("{log}");
                }
                let mut storage = storate.lock().expect("Failed to lock storage");
                storage.add_log(log);
            }
        };
    }
}

fn listen(socket: &mut Socket) -> Result<RsLog> {
    let mut msg = socket.recv().context("Failed to receive message")?;
    let pipe: Pipe = msg.pipe().context("Message missing pipe")?;
    let ip = pipe
        .get_opt::<RemAddr>()
        .context("Failed to get remote address")?
        .to_string();
    let buf: Vec<u8> = msg.as_slice().to_vec();
    let log = flatbuffers::root::<Log>(&buf).context("Failed to deserialize log message")?;
    let log: RsLog = RsLog::from(log, ip);
    Ok(log)
}
