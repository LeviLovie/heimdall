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
        .with_app_name("many")
        .with_version("1.0.0")
        .build()
        .context("Failed to build logger")?;

    for i in 0..1_000_000 {
        log!("Log", "id" => i);
    }

    println!("Log messages sent successfully.");
    Ok(())
}
