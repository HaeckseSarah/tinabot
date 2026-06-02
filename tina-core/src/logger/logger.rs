use super::LogLevel;

pub struct Logger {
    level: LogLevel,
    log_file: Option<std::sync::Mutex<std::fs::File>>,
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
            let prefix = if self.level == LogLevel::ADHD {
                "Oh look, "
            } else {
                ""
            };

            let formatted = format!("{}{}", prefix, msg);

            // CLI
            if level >= LogLevel::Error {
                eprintln!(
                    "{} [{}{}{}] [{}] {}",
                    timestamp,
                    level.color(),
                    level.name(),
                    reset,
                    target,
                    formatted
                );
            } else {
                println!(
                    "{} [{}{}{}] [{}] {}",
                    timestamp,
                    level.color(),
                    level.name(),
                    reset,
                    target,
                    formatted
                );
            }

            // write to file
            if let Some(mutex) = &self.log_file {
                if let Ok(mut file) = mutex.lock() {
                    use std::io::Write;
                    let _ = writeln!(
                        file,
                        "{} [{}] [{}] {}",
                        timestamp,
                        level.name(),
                        target,
                        formatted
                    );
                }
            }
        }
    }
}
