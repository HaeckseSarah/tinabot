use crate::lua::LuaCallback;
use mlua::RegistryKey;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Registry {
    callbacks: Arc<Mutex<HashMap<String, Vec<LuaCallback>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_callback(
        &self,
        event_pattern: String,
        script_name: String,
        func_key: RegistryKey,
        filter_key: Option<RegistryKey>,
    ) {
        let mut cb_map = self.callbacks.lock().unwrap();
        cb_map
            .entry(event_pattern)
            .or_insert_with(Vec::new)
            .push(LuaCallback {
                script_name,
                function_key: Arc::new(func_key),
                filter_key: filter_key.map(Arc::new),
            });
    }

    pub fn get_callbacks_for(&self, event_type: &str) -> Vec<LuaCallback> {
        let cb_map = self.callbacks.lock().unwrap();
        let mut matched_callbacks = Vec::new();

        for (pattern, list) in cb_map.iter() {
            if Self::match_pattern(pattern, event_type) {
                // Dank Arc können wir die Liste jetzt einfach clonen
                matched_callbacks.extend(list.iter().cloned());
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
}
