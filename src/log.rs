use crate::schemas::log::log::{Context, ContextArgs, Log, LogArgs, Var, VarArgs};

pub mod prelude {
    pub use super::{RsContext, RsLog, RsVar};
}

#[derive(Debug, Clone)]
pub struct RsVar {
    pub key: String,
    pub val: String,
}

#[derive(Debug, Clone)]
pub struct RsContext {
    pub pid: u32,
    pub machine: String,
    pub os: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct RsLog {
    pub msg: String,
    pub ts: u64,
    pub context: RsContext,
    pub vars: Vec<RsVar>,
}

impl RsLog {
    pub fn new(msg: String, vars: Vec<(String, String)>, ts: u64, context: RsContext) -> Self {
        let vars = vars
            .into_iter()
            .map(|(key, val)| RsVar { key, val })
            .collect();

        Self {
            msg: msg.into(),
            ts,
            context,
            vars,
        }
    }

    pub fn build<'a>(self) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);

        let msg = builder.create_string(&self.msg);
        let vars = self
            .vars
            .iter()
            .map(|var| {
                let key_string = builder.create_string(&var.key);
                let val_string = builder.create_string(&var.val);
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
        let machine_string = builder.create_string(&self.context.machine);
        let os_string = builder.create_string(&self.context.os);
        let version_string = builder.create_string(&self.context.version);
        let context = Context::create(
            &mut builder,
            &ContextArgs {
                pid: self.context.pid,
                machine: Some(machine_string),
                os: Some(os_string),
                version: Some(version_string),
            },
        );
        let log_offset = Log::create(
            &mut builder,
            &LogArgs {
                msg: Some(msg),
                ts: self.ts as i64,
                context: Some(context),
                vars: Some(vars_array),
            },
        );

        builder.finish(log_offset, None);
        builder.finished_data().to_vec()
    }
}

impl From<Log<'_>> for RsLog {
    fn from(log: Log<'_>) -> Self {
        let vars = log
            .vars()
            .unwrap_or_default()
            .iter()
            .map(|var| RsVar {
                key: var.key().unwrap_or("").to_string(),
                val: var.val().unwrap_or("").to_string(),
            })
            .collect();
        Self {
            msg: log.msg().unwrap_or("").to_string(),
            ts: log.ts() as u64,
            context: RsContext {
                pid: log.context().map_or(0, |ctx| ctx.pid()),
                machine: log
                    .context()
                    .and_then(|ctx| ctx.machine())
                    .unwrap_or("")
                    .to_string(),
                os: log
                    .context()
                    .and_then(|ctx| ctx.os())
                    .unwrap_or("")
                    .to_string(),
                version: log
                    .context()
                    .and_then(|ctx| ctx.version())
                    .unwrap_or("")
                    .to_string(),
            },
            vars,
        }
    }
}

impl std::fmt::Display for RsLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(self.ts as i64)
            .unwrap_or_else(|| chrono::Utc::now());
        let vars_str = self
            .vars
            .iter()
            .map(|var| format!("{}={}", var.key, var.val))
            .collect::<Vec<_>>()
            .join(", ");
        let version_str = if self.context.version.len() > 0 {
            format!(" version={}", self.context.version)
        } else {
            String::new()
        };
        let context_str = format!("pid={}{}", self.context.pid, version_str);

        write!(
            f,
            "{} {}: {} {}",
            timestamp, context_str, self.msg, vars_str
        )
    }
}
