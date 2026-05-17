use crate::logger::LogLevel;
use crate::{Config, Logger};
use mlua::{Lua, Table};
use std::path::PathBuf;
use std::sync::Arc;
use tina_plugin_api::ScriptRegistry;
use walkdir::WalkDir;

pub struct LuaWrapper {
    lua: Lua,
    config: Arc<Config>,
    logger: Arc<Logger>,
}

impl LuaWrapper {
    pub fn new(config: Arc<Config>, logger: Arc<Logger>) -> Self {
        let lua = Lua::new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()).unwrap();
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

    pub fn test_lua(&self) -> Result<(), Box<dyn std::error::Error>> {
        let test_script = r#"
            p.dummy.ping("from lua with love <3")
        "#;

        if let Err(e) = self.lua.load(test_script).exec() {
            let error_msg = format!("Script-Error: {}", e);
            self.logger.log(LogLevel::Error, "LUA", &error_msg);
        }

        Ok(())
    }

    pub fn load_scripts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let script_path = self
            .config
            .get("lua.script_path")
            .ok_or_else(|| "could not read lua.script_path from config!")?;
        let path = PathBuf::from(script_path);

        if !path.exists() {
            return Err(format!("Script Folder not found: {:?}", path).into());
        }

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("lua") {
                let content = std::fs::read_to_string(entry.path())?;
                self.lua.load(&content).exec()?;
                self.logger.log(
                    LogLevel::Info,
                    "LuaPlugin",
                    &format!("script loaded: {:?}", entry.path().file_name().unwrap()),
                );
            }
        }
        Ok(())
    }
}
