//! # Tina Plugin API
//!
//! This crate contains the interfaces, required to build plugins for TinaBot.

pub mod event;
pub mod plugin;
pub mod scripting;

// Re-export types for a clean, API footprint
pub use event::{Event, EventValue};
pub use plugin::{EventTx, LogFn, Plugin, PluginConfig};
pub use scripting::ScriptRegistry;
