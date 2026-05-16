#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warn => "WARN ",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn color(&self) -> &str {
        match self {
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
            0 => Ok(LogLevel::Debug),
            1 => Ok(LogLevel::Info),
            2 => Ok(LogLevel::Warn),
            3 => Ok(LogLevel::Error),
            _ => Err(format!("Unbekanntes Log-Level: {}", value)),
        }
    }
}

pub struct Logger {
    level: LogLevel,
    log_file: Option<std::sync::Mutex<std::fs::File>>, // thread-safe
}
impl Logger {
    pub fn new(level_str: &str, file_path: Option<String>) -> Self {
        let log_file = file_path.and_then(|path| {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
                .map(std::sync::Mutex::new)
        });

        Self {
            level: LogLevel::from_str(level_str),
            log_file,
        }
    }

    pub fn log(&self, level: LogLevel, target: &str, msg: &str) {
        if level >= self.level {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let reset = "\x1b[0m";

            // Konsole
            println!(
                "{} [{}{}{}] [{}] {}",
                timestamp,
                level.color(),
                level.name(),
                reset,
                target,
                msg
            );

            // Datei
            if let Some(mutex) = &self.log_file {
                if let Ok(mut file) = mutex.lock() {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "{} [{}] [{}] {}",
                        timestamp,
                        level.name(),
                        target,
                        msg
                    );
                }
            }
        }
    }
}
