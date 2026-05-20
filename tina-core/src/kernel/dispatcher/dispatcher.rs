use super::core_filters::CoreFilterMatcher;
use crate::kernel::dispatcher::QueueConfig;
use crate::kernel::dispatcher::queue::ActionQueue;
use crate::kernel::{BaseRegistry, EventHandlerRegistry};
use crate::logger::{LogLevel, Logger};
use crate::lua::Wrapper as LuaWrapper;
use mlua::{Table, Value};
use std::sync::Arc;
use tina_plugin_api::Event;
use tina_plugin_api::FilterRegistry;
use tokio::sync::{Mutex, mpsc};

pub struct Dispatcher {
    lua_wrapper: Arc<LuaWrapper>,
    logger: Arc<Logger>,
    event_rx: Mutex<mpsc::Receiver<Event>>,
    event_tx: mpsc::Sender<Event>,
    event_handler_registry: Arc<EventHandlerRegistry>,
    filter_registry: Arc<FilterRegistry>,
    queue_registry: Arc<BaseRegistry<ActionQueue>>,
}

impl Dispatcher {
    pub fn new(lua_wrapper: Arc<LuaWrapper>, logger: Arc<Logger>) -> Arc<Self> {
        let (event_tx, event_rx) = mpsc::channel::<Event>(100);
        let dsptchr = Arc::new(Self {
            lua_wrapper: lua_wrapper.clone(),
            logger,
            event_rx: Mutex::new(event_rx),
            event_tx,
            event_handler_registry: Arc::new(EventHandlerRegistry::new()),
            filter_registry: Arc::new(FilterRegistry::new()),
            queue_registry: Arc::new(BaseRegistry::new()),
        });

        lua_wrapper.set_dispatcher(dsptchr.clone());
        dsptchr
    }

    pub fn get_event_tx(&self) -> &mpsc::Sender<Event> {
        &self.event_tx
    }

    pub fn get_lua_wrapper(&self) -> Arc<LuaWrapper> {
        self.lua_wrapper.clone()
    }
    pub fn get_event_handler_registry(&self) -> Arc<EventHandlerRegistry> {
        return self.event_handler_registry.clone();
    }

    pub fn get_filter_registry(&self) -> Arc<FilterRegistry> {
        return self.filter_registry.clone();
    }

    pub fn add_queue(&self, name: String, config: QueueConfig) {
        self.logger.log(
            LogLevel::Debug,
            "Dispatcher",
            &format!("queue '{}' registered", name),
        );
        let q = ActionQueue::new(config.parallel);
        self.queue_registry.add(name, q);
    }

    pub async fn handle_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        let matched_callbacks = self.event_handler_registry.get(&event.event_type).await;
        self.logger.log(
            LogLevel::Debug,
            "Dispatcher",
            &format!(
                "Event '{}' catched. found callbacks {:?}",
                event.event_type, matched_callbacks
            ),
        );

        println!("{:?}", matched_callbacks);

        if matched_callbacks.is_empty() {
            return Ok(());
        }

        let lua_payload = self.lua_wrapper.to_value(&event)?;
        let payload: Table = match &lua_payload {
            Value::Table(t) => t.get("payload").unwrap_or(t.clone()),
            _ => return Ok(()),
        };

        for cb in matched_callbacks {
            if let Some(filter_key) = cb.filter.as_ref() {
                let filter_table: mlua::Table = self.lua_wrapper.registry_value(filter_key)?;
                if !CoreFilterMatcher::eval(&filter_table, &payload, &self.get_filter_registry())? {
                    continue;
                }
            }

            let payload_clone = self.lua_wrapper.create_table()?;
            for pair in payload.pairs::<Value, Value>() {
                let (k, v) = pair?;
                payload_clone.set(k, v)?;
            }

            if let Some(action_queue) = self.queue_registry.get(&cb.queue) {
                action_queue.add(cb.function, payload_clone);
            } else {
                return Err(Box::from(format!("Undefined Queue '{}'", cb.queue)));
            }
        }

        Ok(())
    }

    pub async fn run(&self) {
        let mut event_rx = self.event_rx.lock().await;

        for (_, queue) in self.queue_registry.keys_values() {
            let q = queue.clone();
            let l = self.get_lua_wrapper().clone();
            let log = self.logger.clone();
            tokio::spawn(async move {
                q.run(l, log.clone()).await;
            });
        }

        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    match event.event_type.as_str() {
                        "dummy.shutdown" => {
                            self.logger.log(
                                LogLevel::Info,
                                "Dispatcher",
                                &format!("Received killSignal from: {}", event.source),
                            );
                            break;
                        }
                        _ => {
                            self.logger.log(
                                LogLevel::Debug,
                                "Dispatcher",
                                &format!("Received event: {:?}", event),
                            );
                            let _ = self.handle_event(event).await;
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    self.logger.log(LogLevel::Info, "Dispatcher", "ctrl+c detected. Shuting down...");
                    break; // exit main loop
                }

                else => break,
            }
        }
    }
}
