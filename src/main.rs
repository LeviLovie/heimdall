mod args;
mod http;
mod nng;
mod tui;

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

use heimdall::prelude::*;

#[tokio::main]
async fn main() {
    let result: Result<()> = async {
        let args = args::parse();
        println!("Starting Heimdall server with args: {args:?}");

        let mut handles = vec![];
        let statuses: Arc<Mutex<Statuses>> = Arc::new(Mutex::new(Statuses::new()));
        let storage: Arc<Mutex<Storage>> = Arc::new(Mutex::new(Storage::new()));

        if let Some(nng_port) = args.nng {
            let nng_port = nng_port.unwrap_or(62000);
            let args_clone = args.clone();
            let statuses_clone = statuses.clone();
            let storage_clone = storage.clone();
            handles.push(tokio::spawn(async move {
                statuses_clone
                    .lock()
                    .unwrap()
                    .set(ThreadType::NNG, ThreadStatus::Running);
                let result = nng::receive(args_clone, nng_port, storage_clone)
                    .context("Failed to run NNG server");
                let status = match result {
                    Ok(()) => ThreadStatus::Stopped,
                    Err(e) => ThreadStatus::Failed(format!("{}", e)),
                };
                statuses_clone.lock().unwrap().set(ThreadType::NNG, status);
            }));
        }

        if let Some(http_port) = args.http {
            let http_port = http_port.unwrap_or(62001);
            let args_clone = args.clone();
            let statuses_clone = statuses.clone();
            let storage_clone = storage.clone();
            handles.push(tokio::spawn(async move {
                statuses_clone
                    .lock()
                    .unwrap()
                    .set(ThreadType::HTTP, ThreadStatus::Running);
                let result = nng::receive(args_clone, http_port, storage_clone)
                    .context("Failed to run HTTP server");
                let status = match result {
                    Ok(()) => ThreadStatus::Stopped,
                    Err(e) => ThreadStatus::Failed(format!("{}", e)),
                };
                statuses_clone.lock().unwrap().set(ThreadType::HTTP, status);
            }));
        }

        if args.tui {
            let statuses_clone = statuses.clone();
            let storage_clone = storage.clone();
            handles.push(tokio::spawn(async move {
                statuses_clone
                    .lock()
                    .unwrap()
                    .set(ThreadType::TUI, ThreadStatus::Running);
                let result = tui::start(statuses_clone.clone(), storage_clone)
                    .context("Failed to run HTTP server");
                let status = match result {
                    Ok(()) => ThreadStatus::Stopped,
                    Err(e) => ThreadStatus::Failed(format!("{}", e)),
                };
                statuses_clone.lock().unwrap().set(ThreadType::TUI, status);
            }));
        }

        for h in handles {
            let _ = h.await;
        }

        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}
