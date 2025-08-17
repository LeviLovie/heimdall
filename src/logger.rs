use anyhow::{anyhow, bail, Context, Result};
use nng::Socket;
use std::sync::{Mutex, OnceLock};

use crate::schemas::log::log::{Log, LogArgs, Var, VarArgs};

pub mod prelude {
    pub use super::{global_log, Logger, LoggerBuilder, GLOBAL_LOGGER};
}

pub static GLOBAL_LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

#[derive(Default)]
pub struct LoggerBuilder {
    bind: Option<String>,
}

impl LoggerBuilder {
    pub fn with_bind(mut self, bind: String) -> Self {
        self.bind = Some(bind);
        self
    }

    pub fn with_address_port(mut self, address: &str, port: u16) -> Self {
        self.bind = Some(format!("tcp://{}:{}", address, port));
        self
    }

    pub fn build(self) -> Result<()> {
        let bind = self
            .bind
            .ok_or_else(|| anyhow!("Bind address must be specified"))
            .context("Building Logger")?;
        let socket = Socket::new(nng::Protocol::Push0).context("Failed to create a new socket")?;
        socket
            .dial(&bind)
            .context("Failed to connect to the server")?;

        let logger = Logger {
            _bind: bind,
            socket,
        };
        GLOBAL_LOGGER
            .set(Mutex::new(logger))
            .map_err(|_| anyhow!("Global logger is already set"))?;

        Ok(())
    }
}

pub struct Logger {
    _bind: String,
    socket: Socket,
}

impl Logger {
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::default()
    }

    pub fn log(&self, msg: impl Into<String>, vars: Vec<(String, String)>) -> Result<()> {
        let msg = msg.into();
        let buf = Self::serialize(&msg, vars);
        if let Err(e) = self.socket.send(&buf) {
            bail!("Failed to send message: {:?}", e);
        }
        Ok(())
    }

    fn serialize(msg: &str, vars: Vec<(String, String)>) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);

        let time: i64 = chrono::Utc::now().timestamp_millis() as i64;
        let msg_string = builder.create_string(msg);

        let vars = vars
            .iter()
            .map(|(key, val)| {
                let key_string = builder.create_string(key);
                let val_string = builder.create_string(val);
                Var::create(
                    &mut builder,
                    &VarArgs {
                        key: Some(key_string),
                        val: Some(val_string),
                    },
                )
            })
            .collect::<Vec<_>>();
        let vars_array = builder.create_vector(&vars);

        let log_entry = Log::create(
            &mut builder,
            &LogArgs {
                msg: Some(msg_string),
                ts: time,
                vars: Some(vars_array),
            },
        );

        builder.finish(log_entry, None);
        builder.finished_data().to_vec()
    }
}

pub fn global_log(msg: impl Into<String>, vars: Vec<(String, String)>) -> Result<()> {
    let logger = &GLOBAL_LOGGER
        .get()
        .ok_or_else(|| anyhow!("Global logger is not initialized"))?;
    logger
        .lock()
        .map_err(|_| anyhow!("Failed to lock global logger"))?
        .log(msg, vars)
        .context("Logging message using global logger")
}
