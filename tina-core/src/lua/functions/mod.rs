use crate::Logger;
use crate::kernel::EventHandlerRegistry;
use mlua::{Lua, Table};
use std::sync::Arc;

pub mod delay;
pub mod on;

pub fn register_all(
    lua: &Lua,
    tina_table: &Table,
    event_registry: Arc<EventHandlerRegistry>,
    logger: Arc<Logger>,
) -> mlua::Result<()> {
    on::register(lua, tina_table, event_registry)?;
    delay::register(lua, tina_table, logger)?;

    Ok(())
}
