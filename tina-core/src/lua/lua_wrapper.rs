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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let globals = self.lua.globals();

        let tina_table: Table = if globals.contains_key("tina")? {
            globals.get("tina")?
        } else {
            let table = self.lua.create_table()?;
            globals.set("tina", table.clone())?;
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
            let mut current_table = tina_table.clone();

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
            tina.dummyPlugin.ping("from lua with love <3")
        "#;

        if let Err(e) = self.lua.load(test_script).exec() {
            let error_msg = format!("Script-Error: {}", e);
            self.logger.log(LogLevel::Error, "LUA", &error_msg);
        }

        Ok(())
    }

    pub fn load_scripts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let scripts_path = self
            .config
            .get("TINA_SCRIPTS_PATH")
            .ok_or_else(|| "could not read TINA_SCRIPTS_PATH from config!")?;
        let path = PathBuf::from(scripts_path);

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
