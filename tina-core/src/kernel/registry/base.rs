use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct BaseRegistry<T: ?Sized> {
    list: Arc<RwLock<HashMap<String, Arc<T>>>>,
}

impl<T: ?Sized + Send + Sync> BaseRegistry<T> {
    pub fn new() -> Self {
        Self {
            list: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add(&self, idx: String, element: Arc<T>) {
        let mut lock = self.list.write().unwrap();
        lock.insert(idx, element);
    }

    pub fn get(&self, idx: &str) -> Option<Arc<T>> {
        let lock = self.list.read().unwrap();
        lock.get(idx).cloned()
    }

    // fixme implement proper iter()
    pub fn keys_values(&self) -> Vec<(String, Arc<T>)> {
        let lock = self.list.read().unwrap();

        lock.iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect()
    }
}

// Implementiere Clone manuell
impl<T: ?Sized> Clone for BaseRegistry<T> {
    fn clone(&self) -> Self {
        Self {
            list: self.list.clone(), // Das klont nur den Arc (Pointer-Inkrement), das ist völlig legal!
        }
    }
}
