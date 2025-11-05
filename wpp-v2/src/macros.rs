// Debug printing macro gated by WPP_DEBUG env var
#[macro_export]
macro_rules! wpp_debug {
    ($($arg:tt)*) => {{
        if std::env::var("WPP_DEBUG").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false) {
            println!($($arg)*);
        }
    }};
}

