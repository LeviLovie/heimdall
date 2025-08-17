use crate::schemas::log::log::Log;

impl std::fmt::Display for Log<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(self.ts())
            .unwrap_or_else(|| chrono::Utc::now());
        let vars = self.vars().unwrap_or_default();
        let vars_str = vars
            .iter()
            .map(|var| {
                let key = var.key().unwrap_or("");
                let val = var.val().unwrap_or("");
                format!("{}={}", key, val)
            })
            .collect::<Vec<_>>()
            .join(", ");

        write!(
            f,
            "{}: {} {}",
            timestamp,
            self.msg().unwrap_or(""),
            vars_str
        )
    }
}
