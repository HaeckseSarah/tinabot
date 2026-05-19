use super::LogLevel;

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
