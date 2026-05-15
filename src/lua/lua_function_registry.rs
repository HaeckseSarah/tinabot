use crate::logger::{LogLevel, Logger};
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell, RwLock, mpsc};

pub struct LuaFunctionDefinition {
    pub name: String,
    pub callback: Arc<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static>,
}

pub struct LuaFunctionRegistry {
    prefix: String,
    fns: Arc<tokio::sync::Mutex<Vec<LuaFunctionDefinition>>>,
}

impl LuaFunctionRegistry {
    // Der Kernel erstellt die Registry spezifisch für dieses eine Plugin
    pub fn new(plugin_id: &str) -> Self {
        Self {
            prefix: plugin_id.to_string(),
            fns: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn register_function<F>(&self, name: &str, callback: F)
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        let full_name: String = format!("{}.{}", self.prefix, name);

        let mut fns = self.fns.lock().await;
        fns.push(LuaFunctionDefinition {
            name: full_name,
            callback: Arc::new(callback),
        });
    }

    pub fn fns(&self) -> Arc<tokio::sync::Mutex<Vec<LuaFunctionDefinition>>> {
        self.fns.clone()
    }
}
