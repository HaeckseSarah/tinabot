use crate::Logger;
use crate::logger::LogLevel;
use mlua::{Lua, Table};
use std::sync::Arc;
use std::time::Duration;

pub fn register(lua: &Lua, tina_table: &Table, logger: Arc<Logger>) -> mlua::Result<()> {
    let delay_function = lua.create_async_function(move |_, msecs: u64| {
        let logger_clone = logger.clone();
        async move {
            logger_clone.log(LogLevel::Info, "Lua", "Script enters sleep...");

            tokio::time::sleep(Duration::from_millis(msecs)).await;

            Ok(())
        }
    })?;

    tina_table.set("delay", delay_function)?;
    Ok(())
}
