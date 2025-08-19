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
        .with_app_name("example-wrap")
        .build()
        .context("Failed to build logger")?;

    log!(
        "Hello, this is gonna trigger line wrapping in the \"info\" panel. I hope it will work fine cause if not ill have to go and fix it!"
    );

    println!("Log message sent successfully.");
    Ok(())
}
