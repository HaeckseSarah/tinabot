use crate::kernel::EventHandlerRegistry;
use mlua::{Lua, RegistryKey, Table, Value};
use std::sync::Arc;

pub fn register(
    lua: &Lua,
    table: &Table,
    event_registry: Arc<EventHandlerRegistry>,
) -> mlua::Result<()> {
    let on_fn = lua.create_function(
        move |lua, (event_type, options, callback): (String, Value, Value)| {
            let mut queue_name = "default".to_string();
            let mut filter_table: Option<RegistryKey> = None;
            let chunk_name = "_fixme_".to_string(); //fixme

            let callback_fn = match callback {
                Value::Function(f) => f,
                _ => {
                    // if 2 arguments t.on("event", function)
                    if let Value::Function(f) = options.clone() {
                        f
                    } else {
                        return Err(mlua::Error::RuntimeError(
                            "Missing callback function in t.on".to_string(),
                        ));
                    }
                }
            };

            if let Value::Table(t) = options.clone() {
                if t.contains_key("queue")? || t.contains_key("filters")? {
                    // Es ist das neue Config-Objekt!
                    if let Ok(q) = t.get("queue") {
                        queue_name = q;
                    }
                    if let Ok(f) = t.get::<mlua::Table>("filters") {
                        filter_table = Some(lua.create_registry_value(f)?);
                    }
                } else {
                    filter_table = Some(lua.create_registry_value(t)?);
                }
            }

            let registry_clone = event_registry.clone();
            let callback = lua.create_registry_value(callback_fn)?;
            tokio::spawn(async move {
                registry_clone
                    .add(&event_type, chunk_name, callback, filter_table, queue_name)
                    .await;
            });

            Ok(())
        },
    )?;

    table.set("on", on_fn)?;
    Ok(())
}
