use crate::Logger;
use mlua::{Lua, Table};
use std::sync::Arc;
use std::time::Duration;

pub fn register(lua: &Lua, tina_table: &Table, _logger: Arc<Logger>) -> mlua::Result<()> {
    let delay_function = lua.create_async_function(move |_, msecs: u64| async move {
        tokio::time::sleep(Duration::from_millis(msecs)).await;
        Ok(())
    })?;

    tina_table.set("delay", delay_function)?;
    Ok(())
}
