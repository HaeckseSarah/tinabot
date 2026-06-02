use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Unsere mLua-unabhängigen Datentypen für den Filter-Vergleich
#[derive(Debug, Clone, PartialEq)]
pub enum ApiValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    None,
}

// Der Typ für unsere Rust-Filter-Closures
pub type FilterFn = Arc<dyn Fn(ApiValue, ApiValue) -> bool + Send + Sync>;

/// Die Registry, die alle nativen Filter-Closures der Plugins hält
#[derive(Default)]
pub struct FilterRegistry {
    // Mapping: "plugin_id" -> Mutex<HashMap<"filter_name", Closure>>
    pub filters: Arc<Mutex<HashMap<String, HashMap<String, FilterFn>>>>,
}

impl FilterRegistry {
    pub fn new() -> Self {
        Self {
            filters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
