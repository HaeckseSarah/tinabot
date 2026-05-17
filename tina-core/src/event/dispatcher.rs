use super::Registry;
use crate::Logger;
use crate::logger::LogLevel;
use mlua::{Lua, LuaSerdeExt, Value};
use std::sync::Arc;
use tina_plugin_api::Event;

pub struct Dispatcher {
    lua: Lua,
    registry: Registry,
    logger: Arc<Logger>,
}

impl Dispatcher {
    pub fn new(lua: Lua, registry: Registry, logger: Arc<Logger>) -> Self {
        Self {
            lua,
            registry,
            logger,
        }
    }

    pub async fn dispatch(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        let callbacks = self
            .registry
            .get_callbacks_for(&self.lua, &event.event_type)?;

        if callbacks.is_empty() {
            return Ok(());
        }

        let lua_payload = self.lua.to_value(&event.payload)?;

        for (script_name, func) in callbacks {
            let payload_clone = match &lua_payload {
                Value::Table(t) => {
                    let copy = self.lua.create_table()?;
                    for pair in t.pairs::<Value, Value>() {
                        let (k, v) = pair?;
                        copy.set(k, v)?;
                    }
                    Value::Table(copy)
                }
                other => other.clone(),
            };

            if let Err(err) = func.call_async::<()>(payload_clone).await {
                self.logger.log(
                    LogLevel::Error,
                    "Lua",
                    &format!(
                        "Event error in script '{}' for event '{}': {}",
                        script_name, event.event_type, err
                    ),
                );
            }
        }

        Ok(())
    }
}
