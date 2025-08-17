#[macro_export]
macro_rules! log {
    ($fmt:expr $(, $key:expr => $val:expr)*) => {{
        let msg = format!($fmt);
        let vars = vec![$(($key.to_string(), $val.to_string())),*];
        $crate::prelude::global_log(msg, vars).unwrap_or_else(|e| {
            eprintln!("Failed to log message: {}", e);
        });
    }};

    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        $crate::prelude::global_log(msg, Vec::new()).unwrap_or_else(|e| {
            eprintln!("Failed to log message: {}", e);
        });
    }};
}
