use super::Sandbox;
use crate::kernel::Dispatcher;
use crate::logger::LogLevel;
use crate::{Config, Logger};
use mlua::{
    FromLua, Function, Lua, LuaOptions, LuaSerdeExt, RegistryKey, StdLib, Table, Thread, Value,
};
use mlua::{IntoLua, Result as LuaResult};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tina_plugin_api::ScriptRegistry;
use tokio::sync::OnceCell;

pub struct Wrapper {
    lua: Lua,
    config: Arc<Config>,
    logger: Arc<Logger>,
    dispatcher: OnceCell<Arc<Dispatcher>>,
}

impl Wrapper {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        // todo: let user choose to use "unsafe" lua?
        let safe_libs = StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(safe_libs, LuaOptions::default()).unwrap();

        Self {
            lua: lua,
            config,
            logger,
            dispatcher: OnceCell::new(),
        }
    }

    pub fn set_dispatcher(&self, dispatcher: Arc<Dispatcher>) {
        let _ = self.dispatcher.set(dispatcher);
        let _ = self.register_core_api();
    }

    fn dispatcher(&self) -> Arc<Dispatcher> {
        self.dispatcher
            .get()
            .expect("dispatcher not initialized!")
            .clone()
    }

    pub fn get_lua(&self) -> Lua {
        self.lua.clone()
    }

    pub fn create_registry<'lua>(&'lua self, namespace: &str) -> ScriptRegistry<'lua> {
        ScriptRegistry::new(namespace, &self.lua)
    }

    pub async fn register_functions<'lua>(
        &self,
        function_registry: ScriptRegistry<'lua>,
        parent: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let globals = self.lua.globals();

        let parent_name = parent.unwrap_or("p");

        let parent_table: Table = if globals.contains_key(parent_name)? {
            globals.get(parent_name)?
        } else {
            let table = self.lua.create_table()?;
            globals.set(parent_name, table.clone())?;
            table
        };

        let functions = function_registry.fns();

        for (name, lua_func) in functions {
            let parts: Vec<&str> = name.split('.').collect();

            self.logger.log(
                LogLevel::Debug,
                "Lua",
                &format!("register lua function: '{}'", name),
            );

            if parts.is_empty() {
                continue;
            }

            let func_name = parts.last().unwrap().to_string();

            let namespace_parts = &parts[..parts.len() - 1];
            let mut current_table = parent_table.clone();

            for &part in namespace_parts {
                let next_table: Table = if current_table.contains_key(part)? {
                    current_table.get(part)?
                } else {
                    let new_table = self.lua.create_table()?;
                    current_table.set(part, new_table.clone())?;
                    new_table
                };

                current_table = next_table;
            }

            current_table.set(func_name.as_str(), lua_func)?;
        }

        Ok(())
    }

    pub async fn load_scripts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let script_path = self
            .config
            .get("lua.script_path")
            .ok_or_else(|| "could not read lua.script_path from config!")?;
        let path = PathBuf::from(script_path);

        if !path.exists() {
            return Err(format!("Script Folder not found: {:?}", path).into());
        }

        let entries = fs::read_dir(&path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // load .lua file in script-folder
                if path.extension().map_or(false, |ext| ext == "lua") {
                    let sandbox = Sandbox::new(&self.lua);
                    sandbox.execute(path).await?;
                }
            } else if path.is_dir() {
                // load main.lua in script/folder, if exists
                let main_lua_path = path.join("main.lua");
                if main_lua_path.exists() {
                    let sandbox = Sandbox::new(&self.lua);
                    sandbox.allow_require(path)?;
                    sandbox.execute(main_lua_path).await?;
                }
            }
        }

        Ok(())
    }

    fn register_core_api(&self) -> LuaResult<()> {
        let globals = self.lua.globals();

        let tina_table: Table = if globals.contains_key("t")? {
            globals.get("t")?
        } else {
            let t = self.lua.create_table()?;
            globals.set("t", t.clone())?;
            t
        };

        super::functions::register_all(
            &self.lua,
            &tina_table,
            self.dispatcher().get_event_handler_registry(),
            self.logger.clone(),
        )?;

        Ok(())
    }

    pub fn to_value<'a, T>(&self, t: &'a T) -> LuaResult<Value>
    where
        T: Serialize + ?Sized,
    {
        self.lua.to_value(t)
    }

    pub fn _create_registry_value<T>(&self, t: T) -> Result<mlua::RegistryKey, mlua::Error>
    where
        T: IntoLua,
    {
        self.lua.create_registry_value(t)
    }

    pub fn registry_value<T>(&self, key: &RegistryKey) -> mlua::Result<T>
    where
        T: FromLua,
    {
        self.lua.registry_value(key)
    }

    pub fn create_table(&self) -> LuaResult<Table> {
        self.lua.create_table()
    }

    pub fn create_thread(&self, func: Function) -> Thread {
        self.get_lua().create_thread(func).unwrap()
    }
}
