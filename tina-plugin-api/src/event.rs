use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Supported types that can be in an event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

/// Event Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier
    pub id: Uuid,
    /// The unique identity string of the plugin that created this event
    pub source: String,
    /// The event type
    pub event_type: String,
    /// Optional key-value payload
    pub payload: Option<HashMap<String, EventValue>>,
}
