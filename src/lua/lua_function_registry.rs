use crate::logger::{LogLevel, Logger};
use mlua::{FromLuaMulti, Function, IntoLuaMulti, Lua};

pub struct LuaFunctionRegistry<'lua> {
    prefix: String,
    lua: &'lua Lua,
    fns: Vec<(String, Function)>,
}

impl<'lua> LuaFunctionRegistry<'lua> {
    // Der Kernel erstellt die Registry spezifisch für dieses eine Plugin
    pub fn new(plugin_id: &str, lua: &'lua Lua) -> Self {
        Self {
            prefix: plugin_id.to_string(),
            lua,
            fns: Vec::new(),
        }
    }

    pub fn register_function<F, A, R>(&mut self, name: &str, callback: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&Lua, A) -> mlua::Result<R> + Send + Sync + 'static,
    {
        let full_name = format!("{}.{}", self.prefix, name);

        let lua_func = self
            .lua
            .create_function(callback)
            .expect("Error while creating lua function");

        self.fns.push((full_name, lua_func));
    }

    pub fn fns(&self) -> Vec<(String, Function)> {
        self.fns.clone()
    }
}
