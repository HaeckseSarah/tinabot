use mlua::{FromLuaMulti, Function, IntoLuaMulti, Lua};

/// A registry acting as bridge to add Rust functions to the Lua runtime
pub struct ScriptRegistry<'lua> {
    pub plugin_id: String,
    lua: &'lua Lua,
    fns: Vec<(String, Function)>,
}

impl<'lua> ScriptRegistry<'lua> {
    /// Creates a new `ScriptRegistry` for a specific plugin
    pub fn new(plugin_id: &str, lua: &'lua Lua) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            lua,
            fns: Vec::new(),
        }
    }

    /// Register a Rust function to Lua in an plugin specific namespace
    pub fn register_function<F, A, R>(&mut self, name: &str, callback: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&Lua, A) -> mlua::Result<R> + Send + Sync + 'static,
    {
        let lua_func = self
            .lua
            .create_function(callback)
            .expect("Error while creating lua function");

        // Namespace functions as 'plugin_id.function_name' to prevent collisions
        let full_name = format!("{}.{}", self.plugin_id, name);
        self.fns.push((full_name, lua_func));
    }

    /// Consumes or references the collected function bindings for ingestion into the Lua VM.
    pub fn fns(&self) -> Vec<(String, Function)> {
        self.fns.clone()
    }
}
