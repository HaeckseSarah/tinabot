use super::functions;
use crate::event::Registry;
use crate::logger::LogLevel;
use crate::lua::Sandbox;
use crate::{Config, Logger};
use mlua::{Function, Lua, LuaOptions, RegistryKey, StdLib, Table, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tina_plugin_api::ScriptRegistry;

pub struct LuaCallback {
    pub script_name: String,
    pub function_key: RegistryKey,
}

pub struct Wrapper {
    lua: Lua,
    config: Arc<Config>,
    logger: Arc<Logger>,
    event_registry: Registry,
}

impl Wrapper {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        let safe_libs = StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(safe_libs, LuaOptions::default()).unwrap();
        let event_registry = Registry::new();

        let wrapper = Self {
            lua: lua,
            config,
            logger,
            event_registry,
        };

        wrapper.register_core_api().unwrap();
        wrapper
    }

    pub fn event_registry(&self) -> Registry {
        self.event_registry.clone()
    }

    pub fn lua_instance(&self) -> Lua {
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
                LogLevel::Info,
                "Lua",
                &format!("register lua function: '{}'", name),
            );

            if parts.is_empty() {
                continue;
            }

            let func_name = parts.last().unwrap().to_string();

            // get/build namespace
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

    fn register_core_api(&self) -> mlua::Result<()> {
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
            self.event_registry.clone(),
            self.logger.clone(),
        )?;

        Ok(())
    }
}
