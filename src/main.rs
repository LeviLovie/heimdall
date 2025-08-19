mod args;
mod data;
mod http;
mod nng;
mod tui;

use anyhow::{Context, Result};
use tokio::task::JoinHandle;
use std::sync::{Arc, Mutex};

use heimdall::prelude::*;
use data::Data;

fn start_thread(data: Arc<Mutex<Data>>, thread_type: ThreadType, closure: impl FnOnce(Arc<Mutex<Data>>) -> Result<()> + Send + 'static) -> JoinHandle<()> {
    let data_clone = data.clone();
    tokio::spawn(async move {
        data_clone.lock().unwrap().statuses.set(
            thread_type.clone(),
            ThreadStatus::Running,
        );
        let result = closure(data_clone.clone());
        data_clone.lock().unwrap().statuses.set(
            thread_type.clone(),
            match result {
                Ok(()) => ThreadStatus::Stopped,
                Err(e) => ThreadStatus::Failed(format!("{}", e)),
            },
        );
    })
}

#[tokio::main]
async fn main() {
    let result: Result<()> = async {
        let args = args::parse();
        let data: Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::new(args.clone(), Statuses::new(), Storage::new())));
        let mut handles = vec![];

        if let Some(nng_port) = args.nng {
            handles.push(start_thread(data.clone(), ThreadType::NNG, move |data| -> Result<()> {
                nng::receive(data, nng_port.unwrap_or(62000)).context("Failed to run NNG server")
            }));
        }

        if let Some(http_port) = args.http {
            handles.push(start_thread(data.clone(), ThreadType::HTTP, move |data| -> Result<()> {
                http::receive(data, http_port.unwrap_or(62001))
                    .context("Failed to run HTTP server")
            }));
        }

        if args.tui {
            handles.push(start_thread(data.clone(), ThreadType::TUI, move |data| -> Result<()> {
                tui::start(data)
                    .context("Failed to run HTTP server")
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
