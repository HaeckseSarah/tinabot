use super::{ApiValue, FilterRegistry};
use crate::{Event, EventTx, EventValue, LogFn, PluginConfig};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct PluginContext {
    plugin_id: String,
    config: PluginConfig,
    event_tx: EventTx,
    log_fn: LogFn,
    filter_registry: Arc<FilterRegistry>,
}

impl PluginContext {
    pub fn new(
        plugin_id: &str,
        config: PluginConfig,
        event_tx: EventTx,
        log_fn: LogFn,
        filter_registry: Arc<FilterRegistry>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.to_lowercase(),
            config,
            event_tx,
            log_fn,
            filter_registry,
        }
    }

    pub async fn send_event(&self, event_type: &str, payload: Option<HashMap<String, EventValue>>) {
        // Hier schweißen wir die Plugin-ID und den Event-Namen untrennbar zusammen
        let prefixed_type = format!("{}.{}", self.plugin_id, event_type);

        let event = Event {
            id: Uuid::new_v4(),
            source: self.plugin_id.clone(),
            event_type: prefixed_type,
            payload,
        };

        let _ = self.event_tx.send(event).await;
    }

    pub fn log(&self, level: i32, message: &str) {
        (self.log_fn)(&level, &self.plugin_id, message);
    }
    pub fn log_error(&self, msg: &str) {
        self.log(3, msg)
    }
    pub fn log_warn(&self, msg: &str) {
        self.log(2, msg)
    }
    pub fn log_info(&self, msg: &str) {
        self.log(1, msg)
    }
    pub fn log_debug(&self, msg: &str) {
        self.log(0, msg)
    }

    pub fn config_get(&self, key: &str) -> Option<String> {
        self.config.get(key)
    }

    pub fn register_filter<F>(&self, name: &str, filter_fn: F)
    where
        F: Fn(ApiValue, ApiValue) -> bool + Send + Sync + 'static,
    {
        let mut all_filters = self.filter_registry.filters.lock().unwrap();
        let plugin_filters = all_filters
            .entry(self.plugin_id.clone())
            .or_insert_with(HashMap::new);

        plugin_filters.insert(name.to_string(), Arc::new(filter_fn));
    }
}
