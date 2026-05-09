/////////////////////////////////////////////////////
// Logging
/////////////////////////////////////////////////////
pub enum LogLevel 
{
    Trace,
    Info,
    Warn,
    Error,
}

macro_rules! log {
    ($level:expr, $fmt:literal $(, $arg:expr)*) => {{
        use colored::Colorize;
        use crate::logging::LogLevel;

        let now = chrono::Local::now().format("%H:%M:%S");

        match $level
        {
            LogLevel::Trace => println!("{}", format!("[{}] [TRACE] - {}", now, format_args!($fmt $(, $arg)*))),
            LogLevel::Info  => println!("{}", format!("[{}] [INFO]  - {}", now, format_args!($fmt $(, $arg)*)).green()),
            LogLevel::Warn  => println!("{}", format!("[{}] [WARN]  - {}", now, format_args!($fmt $(, $arg)*)).yellow()),
            LogLevel::Error => println!("{}", format!("[{}] [ERROR] - {}", now, format_args!($fmt $(, $arg)*)).red()),
        };
    }};
}

#[macro_export] macro_rules! trace { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(LogLevel::Trace, $fmt $(, $arg)*) }}; }
#[macro_export] macro_rules! info  { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(logging::LogLevel::Info,  $fmt $(, $arg)*) }}; }
#[macro_export] macro_rules! warning  { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(logging::LogLevel::Warn,  $fmt $(, $arg)*) }}; }
#[macro_export] macro_rules! error { ($fmt:literal $(, $arg:expr)*) => {{ use crate::logging::LogLevel; log!(logging::LogLevel::Error, $fmt $(, $arg)*) }}; }

pub use trace;
pub use info;
pub use warning;
pub use error;