use crate::Config;
use std::sync::Arc;
use tina_plugin_api::Plugin;

use dummy_plugin::DummyPlugin;
use tokio_util::sync::CancellationToken;
use twitch_plugin::TwitchPlugin;

pub struct PluginHelper {}
impl PluginHelper {
    pub fn get_enabled_plugins(config: Arc<Config>) -> Vec<String> {
        config.get_array("plugins.enabled")
    }

    pub fn create_plugin_instance(
        name: &str,
        cancellation_token: CancellationToken,
    ) -> Option<Arc<dyn Plugin>> {
        match name.to_lowercase().trim() {
            "dummy" => Some(Arc::new(DummyPlugin::new(cancellation_token.clone()))),
            "twitch" => Some(Arc::new(TwitchPlugin::new(cancellation_token.clone()))),
            // "obs" => Some(Arc::new(ObsPlugin::new())),
            _ => None,
        }
    }
}
