use super::base::BaseRegistry;
use mlua::RegistryKey;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct EventHandler {
    pub script_name: String, //todo
    pub function: Arc<RegistryKey>,
    pub filter: Arc<Option<RegistryKey>>,
    pub queue: String,
}

#[derive(Clone)]
pub struct EventHandlerRegistry {
    inner: BaseRegistry<Mutex<Vec<EventHandler>>>,
}

impl EventHandlerRegistry {
    pub fn new() -> Self {
        Self {
            inner: BaseRegistry::new(),
        }
    }

    pub async fn get(&self, event_pattern: &str) -> Vec<EventHandler> {
        let mut matched_callbacks = Vec::new();

        for (pattern, list) in self.inner.keys_values() {
            if Self::match_pattern(&pattern, event_pattern) {
                let lock = list.lock().await;
                matched_callbacks.extend(lock.clone());
            }
        }

        matched_callbacks
    }

    fn match_pattern(pattern: &str, event_type: &str) -> bool {
        if pattern == event_type || pattern == "*" {
            return true;
        }
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return event_type.starts_with(parts[0]) && event_type.ends_with(parts[1]);
            }
        }
        false
    }

    pub async fn add(
        &self,
        event_pattern: &str,
        script_name: String,
        function: RegistryKey,
        filter: Option<RegistryKey>,
        queue: String,
    ) {
        let mutex_vec = if let Some(existing_mutex) = self.inner.get(&event_pattern) {
            existing_mutex
        } else {
            let new_mutex = Arc::new(Mutex::new(Vec::new()));
            self.inner.add(event_pattern.to_string(), new_mutex.clone());
            new_mutex
        };

        let mut lock = mutex_vec.lock().await;
        lock.push(EventHandler {
            script_name,
            function: Arc::new(function),
            filter: Arc::new(filter),
            queue,
        });
    }
}
