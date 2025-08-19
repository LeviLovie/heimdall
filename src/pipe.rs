use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Local};
use serde_json::Value;
use std::io::{self, BufRead};

use crate::args::PipeArgs;
use heimdall::{log, prelude::*};

pub fn pipe(args: PipeArgs) -> Result<()> {
    Logger::builder()
        .with_address_port(&args.address, args.port)
        .with_app_name("piper")
        .with_version(env!("CARGO_PKG_VERSION"))
        .build()
        .context("Failed to build logger")?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                if let Err(e) = pipe_line(l.clone(), &args) {
                    eprintln!("Error processing line: {}", e);
                    log!("{}", l);
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    Ok(())
}

fn pipe_line(line: String, args: &PipeArgs) -> Result<()> {
    if args.json {
        let json_log: Value = serde_json::from_str(&line).context("Failed to parse JSON line")?;
        let mut timestamp = json_log["timestamp"].as_str().clone();
        if timestamp.is_none() {
            timestamp = json_log["ts"].as_str().clone()
        }
        if timestamp.is_none() {
            timestamp = json_log["time"].as_str().clone();
        }
        let ts: DateTime<FixedOffset> = if let Some(timestamp) = timestamp {
            DateTime::parse_from_rfc3339(timestamp)
                .context("Failed to parse timestamp as a rfc3339 string")?
        } else {
            println!("No timestamp found in JSON line, using current time.");
            Local::now().into()
        };
        global_log(ts, line, Vec::new()).context("Failed to log JSON line")?;
    } else {
        log!("{}", line);
    }
    Ok(())
}
