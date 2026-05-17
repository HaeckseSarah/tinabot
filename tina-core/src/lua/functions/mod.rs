use crate::Logger;
use crate::event::Registry;
use mlua::{Lua, Table};
use std::sync::Arc;

pub mod delay;
pub mod on;

pub fn register_all(
    lua: &Lua,
    tina_table: &Table,
    event_registry: Registry,
    logger: Arc<Logger>,
) -> mlua::Result<()> {
    on::register(lua, tina_table, event_registry)?;
    delay::register(lua, tina_table, logger)?;

    Ok(())
}
