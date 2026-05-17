use crate::logger::LogLevel;
use crate::lua::Sandbox;
use crate::{Config, Logger};
use mlua::{Lua, LuaOptions, StdLib, Table};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tina_plugin_api::ScriptRegistry;

pub struct Wrapper {
    lua: Lua,
    config: Arc<Config>,
    logger: Arc<Logger>,
}

impl Wrapper {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        let safe_libs = StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(safe_libs, LuaOptions::default()).unwrap();
        Self {
            lua: lua,
            config,
            logger,
        }
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

        let parent_name = parent.unwrap_or("tina");

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
                // REGEL 1: Jede .lua Datei direkt im /scripts Ordner kriegt eine strikte Sandbox
                if path.extension().map_or(false, |ext| ext == "lua") {
                    let sandbox = Sandbox::new(&self.lua);
                    sandbox.execute(path).await?;
                }
            } else if path.is_dir() {
                // REGEL 2: Bei Ordnern suchen wir NUR nach der main.lua
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
}
