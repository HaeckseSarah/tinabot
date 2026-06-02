#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    ADHD = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "adhd" => LogLevel::ADHD,
            "debug" => LogLevel::Debug,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            LogLevel::ADHD => "ADHD ",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warn => "WARN ",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            LogLevel::ADHD => "\x1b[36m", // cyan
            LogLevel::Debug => "\x1b[35m", // purple
            LogLevel::Info => "\x1b[32m",  // green
            LogLevel::Warn => "\x1b[33m",  // yellow
            LogLevel::Error => "\x1b[31m", // red
        }
    }
}

impl TryFrom<i32> for LogLevel {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, String> {
        match value {
            0 => Ok(LogLevel::ADHD),
            1 => Ok(LogLevel::Debug),
            2 => Ok(LogLevel::Info),
            3 => Ok(LogLevel::Warn),
            4 => Ok(LogLevel::Error),
            _ => Err(format!("Unbekanntes Log-Level: {}", value)),
        }
    }
}
