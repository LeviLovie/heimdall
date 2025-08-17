use anyhow::{Context, Result};
use nng::{Protocol, Socket};

use heimdall::deserialize_log;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let mut socket = Socket::new(Protocol::Pull0).context("Failed to create a new socket")?;
    socket
        .listen("tcp://127.0.0.1:62000")
        .context("Failed to bind socket to address")?;
    println!("Listening for messages on tcp://127.0.0.1:62000");

    loop {
        if let Err(e) = recieve(&mut socket) {
            println!("Error: {:?}", e.context("Failed to recive message"));
        }
    }
}

fn recieve(socket: &mut Socket) -> Result<()> {
    let msg = socket.recv().context("Failed to receive message")?;
    let log = deserialize_log(&msg).context("Failed to deserialize log message")?;
    println!("Received log: {:?}", log);

    Ok(())
}
