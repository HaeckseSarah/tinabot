//! # Tina Plugin API
//!
//! This crate contains the interfaces, required to build plugins for TinaBot.

mod context;
mod event;
mod filter;
mod plugin;
mod scripting;

// Re-export types for a clean, API footprint
pub use context::PluginContext;
pub use event::{Event, EventValue};
pub use filter::{ApiValue, FilterFn, FilterRegistry};
pub use plugin::{EventTx, LogFn, Plugin, PluginConfig};
pub use scripting::ScriptRegistry;
