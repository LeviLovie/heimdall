#[macro_export]
macro_rules! log {
    ( $($arg:tt)* ) => {{
        let msg = format!($($arg)*);
        $crate::prelude::global_log(msg).unwrap_or_else(|e| {
            eprintln!("Failed to log message: {}", e);
        });
    }};
}
