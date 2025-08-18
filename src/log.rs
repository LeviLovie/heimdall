use chrono::{DateTime, FixedOffset};

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
    pub app: String,
    pub pid: u32,
    pub machine: String,
    pub os: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct RsLog {
    pub msg: String,
    pub ts: DateTime<FixedOffset>,
    pub context: RsContext,
    pub vars: Vec<RsVar>,
}

impl RsLog {
    pub fn new(
        msg: String,
        vars: Vec<(String, String)>,
        ts: DateTime<FixedOffset>,
        context: RsContext,
    ) -> Self {
        let vars = vars
            .into_iter()
            .map(|(key, val)| RsVar { key, val })
            .collect();

        Self {
            msg,
            ts,
            context,
            vars,
        }
    }

    pub fn build(self) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);

        let ts_string =
            builder.create_string(&self.ts.to_rfc3339_opts(chrono::SecondsFormat::Micros, true));
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
        let app_string = builder.create_string(&self.context.app);
        let machine_string = builder.create_string(&self.context.machine);
        let os_string = builder.create_string(&self.context.os);
        let version_string = builder.create_string(&self.context.version);
        let context = Context::create(
            &mut builder,
            &ContextArgs {
                app: Some(app_string),
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
                ts: Some(ts_string),
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
        let ts: DateTime<FixedOffset> =
            DateTime::parse_from_rfc3339(log.ts().expect("Log is missing a timestamp"))
                .expect("Failed to parse timestamp");
        let context = log.context();
        Self {
            msg: log.msg().unwrap_or("").to_string(),
            ts,
            context: RsContext {
                app: context.and_then(|ctx| ctx.app()).unwrap_or("").to_string(),
                pid: context.map(|ctx| ctx.pid()).unwrap_or(0),
                machine: context
                    .and_then(|ctx| ctx.machine())
                    .unwrap_or("")
                    .to_string(),
                os: context.and_then(|ctx| ctx.os()).unwrap_or("").to_string(),
                version: context
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
        let vars_str = self
            .vars
            .iter()
            .map(|var| format!("{}={}", var.key, var.val))
            .collect::<Vec<_>>()
            .join(", ");
        let version_str = if !self.context.version.is_empty() {
            format!(" version={}", self.context.version)
        } else {
            String::new()
        };
        let context_str = format!("pid={}{}", self.context.pid, version_str);

        write!(f, "{} {}: {} {}", self.ts, context_str, self.msg, vars_str)
    }
}
