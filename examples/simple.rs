use anyhow::{Context, Result};
use heimdall::{log, prelude::*};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    Logger::builder()
        .with_address_port("127.0.0.1", 62000)
        .build()
        .context("Failed to build logger")?;

    log!("Hello, world!");

    println!("Log message sent successfully.");
    Ok(())
}
