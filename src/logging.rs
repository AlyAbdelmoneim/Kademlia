use chrono::Utc;
use std::{fmt, panic::Location, sync::OnceLock};

pub trait Logging {
    #[track_caller]
    fn log(&self, args: fmt::Arguments);
    #[track_caller]
    fn warn(&self, args: fmt::Arguments);
    #[track_caller]
    fn error(&self, args: fmt::Arguments);

    #[track_caller]
    fn format_message(level: &str, args: fmt::Arguments) -> String {
        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        let caller_location = Location::caller();
        let file = caller_location.file();
        let line = caller_location.line();
        format!("[{}][{}][{}:{}] {}", timestamp, level, file, line, args)
    }
}

pub struct ConsoleLogging;

impl ConsoleLogging {
    pub fn new() -> Self {
        Self {}
    }
}

impl Logging for ConsoleLogging {
    fn log(&self, args: fmt::Arguments) {
        println!("{}", Self::format_message("INFO", args));
    }

    fn warn(&self, args: fmt::Arguments) {
        println!("{}", Self::format_message("WARN", args));
    }

    fn error(&self, args: fmt::Arguments) {
        eprintln!("{}", Self::format_message("ERROR", args));
    }
}

pub struct LoggingFactory;

impl LoggingFactory {
    pub fn logger() -> &'static impl Logging {
        static LOGGER: OnceLock<ConsoleLogging> = OnceLock::new();
        LOGGER.get_or_init(|| ConsoleLogging::new())
    }
}


#[macro_export]
macro_rules! logInfo {
    ($($arg:tt)*) => {{
        use $crate::logging::Logging; 
        $crate::logging::LoggingFactory::logger().log(format_args!($($arg)*));
    }};
}


#[macro_export]
macro_rules! logWarn {
    ($($arg:tt)*) => {{
        use $crate::logging::Logging; 
        $crate::logging::LoggingFactory::logger().warn(format_args!($($arg)*))
    }};
}

#[macro_export]
macro_rules! logError {
    ($($arg:tt)*) => {{
        use $crate::logging::Logging; 
        $crate::logging::LoggingFactory::logger().error(format_args!($($arg)*))
    }};
}