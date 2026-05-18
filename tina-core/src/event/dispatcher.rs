use super::Registry;
use crate::Logger;
use crate::logger::LogLevel;
use crate::lua::CoreFilterMatcher;
use mlua::{Function, Lua, LuaSerdeExt, Table, Value};
use std::sync::Arc;
use tina_plugin_api::{Event, FilterRegistry};

pub struct Dispatcher {
    lua: Lua,
    registry: Registry,
    logger: Arc<Logger>,
    filter_registry: Arc<FilterRegistry>,
}

impl Dispatcher {
    pub fn new(lua: Lua, registry: Registry, logger: Arc<Logger>) -> Self {
        Self {
            lua,
            registry,
            logger,
            filter_registry: std::sync::Arc::new(FilterRegistry::new()),
        }
    }

    pub async fn dispatch(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        let matched_callbacks = self.registry.get_callbacks_for(&event.event_type);
        if matched_callbacks.is_empty() {
            return Ok(());
        }

        let lua_payload = self.lua.to_value(&event)?;
        let payload_table: Table = match &lua_payload {
            Value::Table(t) => t.get("payload").unwrap_or(t.clone()),
            _ => return Ok(()),
        };

        for cb in matched_callbacks {
            let func: Function = self.lua.registry_value(&cb.function_key)?;

            if let Some(filter_key) = &cb.filter_key {
                let filter_table: Table = self.lua.registry_value(filter_key)?;

                // 1. Core-Filter über das ausgelagerte Modul prüfen
                if !CoreFilterMatcher::eval(&filter_table, &payload_table, &self.filter_registry)? {
                    continue;
                }

                // 2. Prüfen, ob Plugin-Filter enthalten sind
                let has_plugin_filters = filter_table.pairs::<usize, Table>().any(|pair| {
                    if let Ok((_, rule)) = pair {
                        let op: String = rule.get(2).unwrap_or_default();
                        op.contains('.')
                    } else {
                        false
                    }
                });

                if has_plugin_filters {
                    let plugin_matched = self
                        .check_plugin_filter(&event.source, &filter_table, &lua_payload)
                        .unwrap_or(false);
                    if !plugin_matched {
                        continue;
                    }
                }
            }

            // Payload isolieren und Callback abfeuern
            let payload_clone = self.lua.create_table()?;
            for pair in payload_table.pairs::<Value, Value>() {
                let (k, v) = pair?;
                payload_clone.set(k, v)?;
            }

            if let Err(err) = func.call_async::<()>(payload_clone).await {
                self.logger.log(
                    LogLevel::Error,
                    "Lua",
                    &format!("Event error in '{}': {}", cb.script_name, err),
                );
            }
        }

        Ok(())
    }

    pub fn get_filter_registry(&self) -> Arc<FilterRegistry> {
        return self.filter_registry.clone();
    }

    ///
    fn check_plugin_filter(
        &self,
        _plugin_id: &str,
        filters: &Table,
        payload: &Value,
    ) -> mlua::Result<bool> {
        let globals = self.lua.globals();
        let tina: Table = globals.get("tina")?;
        let plugins: Table = tina.get("plugins")?;

        for pair in filters.pairs::<usize, Table>() {
            let (_, rule) = pair?;
            let field: Value = rule.get(1)?;
            let operator: String = rule.get(2)?;
            let expected: Value = rule.get(3)?;

            if let Some(dot_idx) = operator.find('.') {
                let (p_id, filter_name) = operator.split_at(dot_idx);
                let filter_name = &filter_name[1..]; // Den Punkt abschneiden

                if let Ok(plugin_meta) = plugins.get::<Table>(p_id) {
                    // Sucht nach der spezifischen Filter-Funktion (z.B. "is_admin")
                    if let Ok(filter_fn) = plugin_meta.get::<Function>(filter_name) {
                        // Wir übergeben dem Plugin: (field_value, expected_value, kompletter_payload)
                        let payload_table = match payload {
                            Value::Table(t) => t.get("payload").unwrap_or(t.clone()),
                            _ => self.lua.create_table()?,
                        };

                        let field_val: Value = match &field {
                            Value::String(s) => {
                                payload_table.get(&*s.to_str()?).unwrap_or(Value::Nil)
                            }
                            _ => Value::Nil,
                        };

                        let matched: bool = filter_fn.call((field_val, expected, payload_table))?;
                        if !matched {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}
