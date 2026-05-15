use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub source: String,                            // e.g. "twitch"
    pub event_type: String,                        // e.g. "susbscription"
    pub payload: Option<serde_json::Value>,        // for lua scripts
    pub metadata: Option<HashMap<String, String>>, // for filters, etc.
}
