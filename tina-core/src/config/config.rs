use crate::cli::Args;
use config::{Config as ConfigBuilder, File};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
pub struct Config {
    cache: config::Config,
}

impl Config {
    pub fn load(args: &Args) -> Self {
        // default config "tina.toml"
        let config_path = args
            .config
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "tina.toml".to_string());

        let mut builder =
            ConfigBuilder::builder().add_source(File::with_name(&config_path).required(false));

        // =================================================================
        // OVERRIDES
        // =================================================================

        if let Ok(env_script_path) = std::env::var("TINA_SCRIPT_PATH") {
            builder = builder
                .set_override("script_path", env_script_path)
                .unwrap();
        }
        if let Some(ref script_path) = args.script_path {
            builder = builder
                .set_override("lua.script_path", script_path.clone())
                .unwrap();
        }

        // cli overrides (-D foo=bar)
        for (key, value) in &args.overrides {
            builder = builder
                .set_override(&key.to_lowercase(), value.clone())
                .unwrap();
        }

        // =================================================================
        let configuration = builder.build().unwrap_or_else(|err| {
            eprintln!("[ERROR] Failed to parse configuration: {}", err);
            std::process::exit(1);
        });

        Self {
            cache: configuration,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get_string(&key.to_lowercase()).ok()
    }

    pub fn get_array(&self, key: &str) -> Vec<String> {
        self.cache
            .get_array(&key.to_lowercase())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.into_string().ok())
            .collect()
    }

    pub fn get_table<T>(&self, key: &str) -> HashMap<String, T>
    where
        T: DeserializeOwned,
    {
        self.cache
            .get::<HashMap<String, T>>(&key.to_lowercase())
            .unwrap_or_default()
    }
}
