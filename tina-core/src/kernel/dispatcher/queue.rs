use crate::Logger;
use crate::logger::LogLevel;
use crate::lua::Wrapper as LuaWrapper;
use mlua::{Function, RegistryKey, Table};
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub struct ScriptTask {
    pub callback: Arc<RegistryKey>,
    pub payload: Table,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueueConfig {
    #[serde(default)] //default false
    pub parallel: bool,
}

pub struct ActionQueue {
    is_parallel: bool,
    tasks: Mutex<VecDeque<ScriptTask>>,
    notify: Notify,
}

impl ActionQueue {
    pub fn new(is_parallel: bool) -> Arc<Self> {
        Arc::new(Self {
            is_parallel,
            tasks: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        })
    }

    pub fn add(&self, callback: Arc<RegistryKey>, payload: Table) {
        let task = ScriptTask {
            callback: callback,
            payload: payload,
        };

        let mut tasks = self.tasks.lock().unwrap();
        tasks.push_back(task);
        self.notify.notify_one();
    }

    pub async fn run(self: Arc<Self>, lua: Arc<LuaWrapper>, logger: Arc<Logger>) {
        loop {
            let log = logger.clone();
            let l = lua.clone();

            let next_task = {
                let mut tasks = self.tasks.lock().unwrap();
                tasks.pop_front()
            };

            if let Some(task) = next_task {
                if self.is_parallel {
                    tokio::spawn(async move {
                        if let Err(_e) = Self::execute(task, l.clone(), log.clone()) {}
                    });
                } else {
                    if let Err(e) = Self::execute(task, l.clone(), log.clone()) {
                        logger.log(LogLevel::Error, "Queue", &format!("Err in queue {:?}", e));
                    }
                }
            } else {
                self.notify.notified().await;
            }
        }
    }

    fn execute(task: ScriptTask, lua: Arc<LuaWrapper>, logger: Arc<Logger>) -> mlua::Result<()> {
        let func: Function = lua.registry_value(&task.callback)?;
        let co = lua.create_thread(func);

        let mut args = mlua::Value::Table(task.payload);

        loop {
            match co.resume::<mlua::Value>(args) {
                Ok(mlua::Value::Nil) => {
                    break;
                }
                Ok(yielded_value) => {
                    args = yielded_value;
                }
                Err(e) => {
                    logger.log(LogLevel::Error, "Queue", &format!("Lua Error: {:?}", e));
                    eprintln!("Lua Error: {:?}", e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn _len(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    pub fn _clear(&self) {
        self.tasks.lock().unwrap().clear();
    }
}
