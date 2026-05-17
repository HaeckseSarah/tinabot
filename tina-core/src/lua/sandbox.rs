use mlua::{Lua, Table, Value};
use std::fs;
use std::path::PathBuf;

pub struct Sandbox<'lua> {
    lua: &'lua Lua,
    env: Table,
}

impl<'lua> Sandbox<'lua> {
    pub fn new(lua: &'lua Lua) -> Sandbox<'lua> {
        Self {
            lua: lua,
            env: Sandbox::build_env(lua).unwrap(),
        }
    }

    pub fn build_env(lua: &Lua) -> Result<Table, Box<dyn std::error::Error>> {
        let globals = lua.globals();
        let sandbox_env = lua.create_table()?;

        let metatable = lua.create_table()?;
        metatable.set("__index", globals)?;
        sandbox_env.set_metatable(Some(metatable))?;

        Ok(sandbox_env)
    }

    pub fn allow_require(&self, base_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.lua.load_std_libs(mlua::StdLib::PACKAGE)?;

        let global_package: Table = self.lua.globals().get("package")?;
        let global_require: mlua::Function = self.lua.globals().get("require")?;

        let base_dir_str = base_dir.to_string_lossy();
        let sandbox_path = format!("{}/?.lua;{}/?/init.lua", base_dir_str, base_dir_str);

        global_package.set("path", sandbox_path)?;
        global_package.set("loaded", self.lua.create_table()?)?; // Leerer Cache für diese Sandbox

        self.env.set("require", global_require)?;
        self.env.set("package", global_package)?;

        self.lua.globals().set("package", Value::Nil)?;
        self.lua.globals().set("require", Value::Nil)?;

        Ok(())
    }

    pub async fn execute(&self, file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if file.is_file() && file.extension().map_or(false, |ext| ext == "lua") {
            let file_name: String = file.file_name().unwrap().to_string_lossy().into_owned();
            let file_content = fs::read_to_string(&file)?;
            let result = self
                .lua
                .load(file_content)
                .set_name(file_name)
                .set_environment(self.env.clone())
                .exec_async()
                .await;

            if let Err(err) = result {
                return Err(
                    format!("Error in {}: {}", file.to_str().unwrap(), err.to_string()).into(),
                );
            }
        } else {
            return Err(format!("File {} not found!", file.to_str().unwrap()).into());
        };
        Ok(())
    }
}
