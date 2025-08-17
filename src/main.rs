mod args;

use anyhow::{Context, Result};
use nng::{Protocol, Socket};

use heimdall::schemas::log::log::Log;

pub fn deserialize_log(buf: &[u8]) -> Result<Log> {
    flatbuffers::root::<Log>(buf).context("Failed to deserialize Log")
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = args::parse();

    receive(&args).context("Failed to start a receiving server")
}

fn receive(args: &args::Args) -> Result<()> {
    let bind = format!("tcp://{}:{}", args.address, args.port);

    let mut socket = Socket::new(Protocol::Pull0).context("Failed to create a new socket")?;
    socket
        .listen(&bind)
        .context("Failed to bind socket to address")?;
    println!("Listening for messages on {}", bind);

    loop {
        if let Err(e) = listen(&mut socket) {
            println!("Error: {:?}", e.context("Failed to recive message"));
        }
    }
}

fn listen(socket: &mut Socket) -> Result<()> {
    let msg = socket.recv().context("Failed to receive message")?;
    let log = deserialize_log(&msg).context("Failed to deserialize log message")?;
    println!("{}", log);
    Ok(())
}
