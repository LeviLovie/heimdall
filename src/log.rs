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
    pub os: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct RsLog {
    pub ts: DateTime<FixedOffset>,
    pub msg: String,
    pub ip: String,
    pub context: RsContext,
    pub vars: Vec<RsVar>,
}

impl RsLog {
    pub fn new(
        ts: DateTime<FixedOffset>,
        msg: String,
        context: RsContext,
        vars: Vec<(String, String)>,
    ) -> Self {
        let vars = vars
            .into_iter()
            .map(|(key, val)| RsVar { key, val })
            .collect();

        Self {
            ts,
            msg,
            ip: String::new(),
            context,
            vars,
        }
    }

    pub fn from(log: Log<'_>, ip: String) -> Self {
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
            ts,
            msg: log.msg().unwrap_or("").to_string(),
            ip,
            context: RsContext {
                app: context.and_then(|ctx| ctx.app()).unwrap_or("").to_string(),
                pid: context.map(|ctx| ctx.pid()).unwrap_or(0),
                os: context.and_then(|ctx| ctx.os()).unwrap_or("").to_string(),
                version: context
                    .and_then(|ctx| ctx.version())
                    .unwrap_or("")
                    .to_string(),
            },
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
        let os_string = builder.create_string(&self.context.os);
        let version_string = builder.create_string(&self.context.version);
        let context = Context::create(
            &mut builder,
            &ContextArgs {
                app: Some(app_string),
                pid: self.context.pid,
                os: Some(os_string),
                version: Some(version_string),
            },
        );
        let log_offset = Log::create(
            &mut builder,
            &LogArgs {
                ts: Some(ts_string),
                msg: Some(msg),
                context: Some(context),
                vars: Some(vars_array),
            },
        );

        builder.finish(log_offset, None);
        builder.finished_data().to_vec()
    }
}

impl std::fmt::Display for RsLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_ts = self.ts.format("%H:%M:%S%.3f");
        let vars_str = self
            .vars
            .iter()
            .map(|var| format!("{}={}", var.key, var.val))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}: {} {}", formatted_ts, self.msg, vars_str)
    }
}
