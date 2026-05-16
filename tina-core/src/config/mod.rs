use crate::cli::Args;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        let _ = dotenvy::from_filename("tina.conf");
        let mut settings: HashMap<String, String> = env::vars().collect();
        for (key, value) in &args.overrides {
            settings.insert(key.clone(), value.clone());
        }

        Self { settings }
    }

    pub fn get(&self, key: &str, default: &str) -> String {
        self.settings
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}
