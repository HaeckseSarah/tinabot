use mlua::{Lua, Table, Value};
use crate::event::Registry;

// Der Funktionsname wurde an deine Modulstruktur (on::register) angepasst
pub fn register(lua: &Lua, t: &Table, event_registry: Registry) -> mlua::Result<()> {
    let registry_clone = event_registry.clone();
    
    let on_fn = lua.create_function(move |lua, (event_type, arg2, arg3): (String, Value, Option<Value>)| {
        // FIX: debug.source() direkt zu String konvertieren
        let chunk_name = "sandbox_script".to_string(); // fixme

        let (filter_key, func_key) = match (arg2, arg3) {
            // Fall 1: Filter-Tabelle + Callback-Funktion
            (Value::Table(filter_table), Some(Value::Function(callback_fn))) => {
                // FIX: Tippfehler behoben
                let f_key = lua.create_registry_value(Value::Table(filter_table))?;
                let c_key = lua.create_registry_value(Value::Function(callback_fn))?;
                (Some(f_key), c_key)
            }
            
            // Fall 2: Nur Callback-Funktion
            (Value::Function(callback_fn), None) => {
                let c_key = lua.create_registry_value(Value::Function(callback_fn))?;
                (None, c_key)
            }
            
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "Invalid arguments to t.on. Expected (string, function) or (string, table, function).".to_string()
                ));
            }
        };

        registry_clone.register_callback(event_type, chunk_name, func_key, filter_key);
        Ok(())
    })?;

    t.set("on", on_fn)?;
    Ok(())
}