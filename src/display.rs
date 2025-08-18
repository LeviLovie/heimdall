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
        let context = self.context();
        let context_str = if let Some(ctx) = context {
            let version_str = if let Some(version) = ctx.version()
                && version.len() > 0
            {
                format!(" version={}", version)
            } else {
                String::new()
            };
            format!("pid={}{}", ctx.pid(), version_str)
        } else {
            String::new()
        };

        write!(
            f,
            "{} {}: {} {}",
            timestamp,
            context_str,
            self.msg().unwrap_or(""),
            vars_str
        )
    }
}
