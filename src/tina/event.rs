use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub source: String,                    // e.g. "twitch"
    pub event_type: String,                // e.g. "susbscription"
    pub payload: serde_json::Value,        // for lua scripts
    pub metadata: HashMap<String, String>, // for filters, etc.
}
