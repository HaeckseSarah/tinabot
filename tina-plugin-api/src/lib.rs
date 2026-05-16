use async_trait::async_trait;
use mlua::{FromLuaMulti, Function, IntoLuaMulti, Lua};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub source: String,     // e.g. "twitch"
    pub event_type: String, // e.g. "susbscription"
    pub payload: Option<HashMap<String, EventValue>>,
}

pub type ConfigLookup = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;
pub type LogFn = Arc<dyn Fn(&i32, &str, &str) + Send + Sync>; // (Level, target, Message)
pub type EventTx = Sender<Event>; // Erstmal simpel als String

#[async_trait]
pub trait Plugin: Send + Sync {
    fn id(&self) -> &str;

    // Das Plugin bekommt beim Booten nur noch die Closures und die mIua Registry (als dynamisches Objekt)
    async fn boot<'lua>(
        &self,
        config_get: ConfigLookup,
        log: LogFn,
        event_tx: EventTx,
        script_registry: &mut ScriptRegistry<'_>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn run(&self);

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct ScriptRegistry<'lua> {
    pub plugin_id: String,
    lua: &'lua Lua,
    fns: Vec<(String, Function)>,
}

impl<'lua> ScriptRegistry<'lua> {
    pub fn new(plugin_id: &str, lua: &'lua Lua) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            lua,
            fns: Vec::new(),
        }
    }

    pub fn register_function<F, A, R>(&mut self, name: &str, callback: F)
    where
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&Lua, A) -> mlua::Result<R> + Send + Sync + 'static,
    {
        let lua_func = self
            .lua
            .create_function(callback)
            .expect("Error while creating lua function");

        let full_name = format!("{}.{}", self.plugin_id, name);
        self.fns.push((full_name, lua_func));
    }

    pub fn fns(&self) -> Vec<(String, Function)> {
        self.fns.clone()
    }
}
