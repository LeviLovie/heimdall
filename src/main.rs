mod args;

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
    let args = args::parse();
    println!("Parsed arguments: {:?}", args);

    match &args.cmd {
        args::Cmd::Receive(recieve_args) => {
            receive(&args, &recieve_args).context("Failed to start a receiving server")
        }
    }
}

fn receive(_args: &args::Args, recieve_args: &args::ReceiveArgs) -> Result<()> {
    let bind = format!("tcp://{}:{}", recieve_args.address, recieve_args.port);

    let mut socket = Socket::new(Protocol::Pull0).context("Failed to create a new socket")?;
    socket
        .listen(&bind)
        .context("Failed to bind socket to address")?;
    println!("Listening for messages on {}", bind);

    loop {
        if let Err(e) = receive_one(&mut socket) {
            println!("Error: {:?}", e.context("Failed to recive message"));
        }
    }
}

fn receive_one(socket: &mut Socket) -> Result<()> {
    let msg = socket.recv().context("Failed to receive message")?;
    let log = deserialize_log(&msg).context("Failed to deserialize log message")?;
    println!("{}", log);
    Ok(())
}
