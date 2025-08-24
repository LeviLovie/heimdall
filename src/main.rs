mod args;
mod data;
mod http;
mod nng;
mod pipe;
mod tui;

use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

use data::Data;
use heimdall::prelude::*;

use crate::args::ServerArgs;

fn start_thread(
    data: Arc<Mutex<Data>>,
    thread_type: ThreadType,
    closure: impl FnOnce(Arc<Mutex<Data>>) -> Result<()> + Send + 'static,
) -> JoinHandle<()> {
    let data_clone = data.clone();
    tokio::spawn(async move {
        data_clone
            .lock()
            .unwrap()
            .statuses
            .set(thread_type.clone(), ThreadStatus::Running);
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

async fn start_server(args: ServerArgs) -> Result<()> {
    let storage = if let Some(path) = args.sqlite.clone() {
        Storage::new_sqlite(path.unwrap_or("logs.sqlite".to_string()))
            .context("Failed to create SQLite storage")?
    } else {
        Storage::new_memory()
    };
    let data: Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::new(
        args.clone(),
        Statuses::new(),
        storage,
    )));
    let mut handles = vec![];

    if let Some(nng_port) = args.nng {
        handles.push(start_thread(
            data.clone(),
            ThreadType::NNG,
            move |data| -> Result<()> {
                nng::receive(data, nng_port.unwrap_or(62000)).context("Failed to run NNG server")
            },
        ));
    }

    if let Some(http_port) = args.http {
        handles.push(start_thread(
            data.clone(),
            ThreadType::HTTP,
            move |data| -> Result<()> {
                http::receive(data, http_port.unwrap_or(62001)).context("Failed to run HTTP server")
            },
        ));
    }

    if args.tui {
        handles.push(start_thread(
            data.clone(),
            ThreadType::TUI,
            move |data| -> Result<()> { tui::start(data).context("Failed to run HTTP server") },
        ));
    }

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let result: Result<()> = async {
        let args = args::parse();

        match args.cmd {
            args::Cmd::Server(server_args) => start_server(server_args)
                .await
                .context("Failed to start server"),
            args::Cmd::Pipe(pipe_args) => pipe::pipe(pipe_args).context("Failed to pipe"),
        }
        .context("Failed to execute command")?;

        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}
