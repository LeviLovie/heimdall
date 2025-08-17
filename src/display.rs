use crate::schemas::log::log::Log;

impl std::fmt::Display for Log<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(self.ts())
            .unwrap_or_else(|| chrono::Utc::now());

        write!(f, "{}: {}", timestamp, self.msg().unwrap_or(""))
    }
}
