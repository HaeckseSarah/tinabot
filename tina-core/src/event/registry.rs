use crate::lua::LuaCallback;
use mlua::{Function, Lua, RegistryKey};
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
        event_type: String,
        script_name: String,
        func_key: RegistryKey,
    ) {
        let mut cb_map = self.callbacks.lock().unwrap();
        cb_map
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(LuaCallback {
                script_name,
                function_key: func_key,
            });
    }

    pub fn get_callbacks_for(
        &self,
        lua: &Lua,
        event_type: &str,
    ) -> mlua::Result<Vec<(String, Function)>> {
        let cb_map: std::sync::MutexGuard<'_, HashMap<String, Vec<LuaCallback>>> =
            self.callbacks.lock().unwrap();

        let mut active_funcs = Vec::new();

        if let Some(list) = cb_map.get(event_type) {
            for cb in list {
                let func: Function = lua.registry_value(&cb.function_key)?;
                active_funcs.push((cb.script_name.clone(), func));
            }
        }

        Ok(active_funcs)
    }
}
