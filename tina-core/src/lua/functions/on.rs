use crate::event::Registry;
use mlua::{Function, Lua, Table};

pub fn register(lua: &Lua, tina_table: &Table, event_registry: Registry) -> mlua::Result<()> {
    let on_function = lua.create_function(move |lua, (event_type, func): (String, Function)| {
        let func_key = lua.create_registry_value(func)?;
        let chunk_name = "sandbox_script".to_string(); // fixme

        event_registry.register_callback(event_type, chunk_name, func_key);
        Ok(())
    })?;

    tina_table.set("on", on_function)?;
    Ok(())
}
