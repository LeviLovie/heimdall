use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use nng::Socket;
use std::sync::{Mutex, OnceLock};

use crate::prelude::{RsContext, RsLog};

pub mod prelude {
    pub use super::{global_log, Logger, LoggerBuilder, GLOBAL_LOGGER};
}

pub static GLOBAL_LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

pub struct LoggerBuilder {
    bind: Option<String>,
    app_name: String,
    version: String,
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self {
            bind: None,
            app_name: "default".to_string(),
            version: "0.0.0".to_string(),
        }
    }
}

impl LoggerBuilder {
    pub fn with_bind(mut self, bind: String) -> Self {
        self.bind = Some(bind);
        self
    }

    pub fn with_address_port(mut self, address: &str, port: u16) -> Self {
        self.bind = Some(format!("tcp://{address}:{port}"));
        self
    }

    pub fn with_app_name(mut self, app_name: &str) -> Self {
        self.app_name = app_name.to_string();
        self
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
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

        let pid = std::process::id();
        let os = Self::get_os();
        let context = RsContext {
            app: self.app_name,
            pid,
            os,
            version: self.version,
        };

        let logger = Logger {
            _bind: bind,
            socket,
            context,
        };
        GLOBAL_LOGGER
            .set(Mutex::new(logger))
            .map_err(|_| anyhow!("Global logger is already set"))?;

        Ok(())
    }

    fn get_os() -> String {
        let kind = sys_info::os_type().unwrap_or_else(|_| "Unknown".to_string());
        let release = sys_info::os_release().unwrap_or_else(|_| "Unknown".to_string());
        format!("{kind} {release}")
    }
}

pub struct Logger {
    _bind: String,
    socket: Socket,
    context: RsContext,
}

impl Logger {
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::default()
    }

    pub fn log(&self, msg: impl Into<String>, vars: Vec<(String, String)>) -> Result<()> {
        let log = RsLog::new(Local::now().into(), msg.into(), self.context.clone(), vars);
        let buf = log.build();

        if let Err(e) = self.socket.send(&buf) {
            bail!("Failed to send message: {:?}", e);
        }
        Ok(())
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
