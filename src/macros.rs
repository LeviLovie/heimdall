#[macro_export]
macro_rules! log {
    ($fmt:expr $(, $key:expr => $val:expr)*) => {{
        let ts = $crate::prelude::current_timestamp();
        let msg = format!($fmt);
        let vars = vec![$(($key.to_string(), $val.to_string())),*];
        $crate::prelude::global_log(ts, msg, vars).unwrap_or_else(|e| {
            eprintln!("Failed to log message: {}", e);
        });
    }};

    ($($arg:tt)*) => {{
        let ts = $crate::prelude::current_timestamp();
        let msg = format!($($arg)*);
        $crate::prelude::global_log(ts, msg, Vec::new()).unwrap_or_else(|e| {
            eprintln!("Failed to log message: {}", e);
        });
    }};
}
