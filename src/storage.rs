use chrono::{DateTime, FixedOffset};
use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::prelude::{RsContext, RsLog, RsVar};

pub mod prelude {
    pub use super::Storage;
}

enum Backend {
    Memory(Vec<(usize, RsLog)>),
    Sqlite(Connection),
}

pub struct Storage {
    backend: Backend,
    pub updated: bool,
}

impl Storage {
    pub fn new_memory() -> Self {
        Self {
            backend: Backend::Memory(Vec::new()),
            updated: true, // Start at updated state so that the renderer fetches all logs
        }
    }

    pub fn new_sqlite(path: impl Into<String>) -> Result<Self> {
        let path_str = path.into();
        let conn = Connection::open(&path_str)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id       INTEGER PRIMARY KEY,
                ts       TEXT NOT NULL,
                msg      TEXT NOT NULL,
                ip       TEXT NOT NULL,
                app      TEXT NOT NULL,
                pid      INTEGER NOT NULL,
                os       TEXT NOT NULL,
                version  TEXT NOT NULL,
                vars     TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self {
            backend: Backend::Sqlite(conn),
            updated: true, // Start at updated state so that the renderer fetches all logs
        })
    }

    pub fn add_log(&mut self, log: RsLog) -> Result<()> {
        match &mut self.backend {
            Backend::Memory(vec) => {
                vec.push((vec.len(), log));
            }
            Backend::Sqlite(conn) => {
                let vars_json: serde_json::Map<String, serde_json::Value> = log
                    .vars
                    .iter()
                    .map(|v| (v.key.clone(), serde_json::Value::String(v.val.clone())))
                    .collect();

                let vars_json_str = serde_json::Value::Object(vars_json).to_string();

                conn.execute(
                    "INSERT INTO logs (ts, msg, ip, app, pid, os, version, vars)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        log.ts.to_rfc3339(),
                        log.msg,
                        log.ip,
                        log.context.app,
                        log.context.pid,
                        log.context.os,
                        log.context.version,
                        vars_json_str
                    ],
                )?;
            }
        }
        self.updated = true;
        Ok(())
    }

    pub fn was_updated(&self) -> bool {
        self.updated
    }

    pub fn logs_amount(&self) -> usize {
        match &self.backend {
            Backend::Memory(vec) => vec.len(),
            Backend::Sqlite(conn) => {
                let mut stmt = conn
                    .prepare("SELECT COUNT(*) FROM logs")
                    .expect("Failed to prepare statement");
                let count: usize = stmt
                    .query_row([], |row| row.get(0))
                    .expect("Failed to execute query");
                count
            }
        }
    }

    pub fn get_log(&self, index: usize) -> Option<RsLog> {
        match &self.backend {
            Backend::Memory(vec) => vec.get(index).map(|(_, log)| log.clone()),
            Backend::Sqlite(conn) => {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, ts, msg, ip, app, pid, os, version, vars
                         FROM logs
                         ORDER BY id DESC
                         LIMIT 1 OFFSET ?1",
                    )
                    .expect("Failed to prepare statement");
                let log_opt = stmt
                    .query_row(params![index as i64], |row| {
                        let ts_str: String = row.get(1)?;
                        let ts: DateTime<FixedOffset> =
                            DateTime::parse_from_rfc3339(&ts_str).unwrap();
                        let vars_str: String = row.get(8)?;
                        let vars_json: serde_json::Value =
                            serde_json::from_str(&vars_str).unwrap_or_default();
                        let vars = vars_json
                            .as_object()
                            .unwrap_or(&serde_json::Map::new())
                            .iter()
                            .map(|(k, v)| RsVar {
                                key: k.clone(),
                                val: v.as_str().unwrap_or("").to_string(),
                            })
                            .collect();
                        Ok(RsLog {
                            ts,
                            msg: row.get(2)?,
                            ip: row.get(3)?,
                            context: RsContext {
                                app: row.get(4)?,
                                pid: row.get(5)?,
                                os: row.get(6)?,
                                version: row.get(7)?,
                            },
                            vars,
                        })
                    })
                    .optional()
                    .expect("Failed to execute query");
                log_opt
            }
        }
    }

    pub fn get_visible_logs(&self, start: usize, amount: usize) -> Result<Vec<(usize, RsLog)>> {
        match &self.backend {
            Backend::Memory(vec) => {
                let total = vec.len();
                let end = total.saturating_sub(start);
                let start_idx = end.saturating_sub(amount);
                Ok(vec[start_idx..end].to_vec())
            }

            // Sqlite backend: query window using LIMIT/OFFSET
            Backend::Sqlite(conn) => {
                let mut stmt = conn.prepare(
                    "SELECT id, ts, msg, ip, app, pid, os, version, vars
                     FROM logs
                     ORDER BY id DESC
                     LIMIT ?1 OFFSET ?2",
                )?;

                let rows = stmt.query_map(params![amount as i64, start as i64], |row| {
                    let id: usize = row.get(0)?;
                    let ts_str: String = row.get(1)?;
                    let ts: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(&ts_str).unwrap();

                    let vars_str: String = row.get(8)?;
                    let vars_json: serde_json::Value =
                        serde_json::from_str(&vars_str).unwrap_or_default();
                    let vars = vars_json
                        .as_object()
                        .unwrap_or(&serde_json::Map::new())
                        .iter()
                        .map(|(k, v)| RsVar {
                            key: k.clone(),
                            val: v.as_str().unwrap_or("").to_string(),
                        })
                        .collect();

                    Ok((
                        id,
                        RsLog {
                            ts,
                            msg: row.get(2)?,
                            ip: row.get(3)?,
                            context: RsContext {
                                app: row.get(4)?,
                                pid: row.get(5)?,
                                os: row.get(6)?,
                                version: row.get(7)?,
                            },
                            vars,
                        },
                    ))
                })?;

                let mut logs = Vec::new();
                for log in rows {
                    logs.push(log?);
                }
                Ok(logs)
            }
        }
    }
}
