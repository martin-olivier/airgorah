use crate::globals::*;
use crate::types::*;

use std::path::Path;

/// Load settings from the config file, otherwise use default settings
pub fn load_settings() {
    if Path::new(CONFIG_PATH).exists() {
        let config = std::fs::read_to_string(CONFIG_PATH).unwrap_or_default();
        let mut settings: Settings = toml::from_str(&config).unwrap_or_default();

        if settings.kill_network_manager && !super::has_dependency("systemctl") {
            settings.kill_network_manager = false;
        }

        log::debug!("settings loaded from '{}'", CONFIG_PATH);

        *SETTINGS.lock().unwrap() = settings;
    }
}

/// Save settings to the config file
pub fn save_settings(mut settings: Settings) {
    if settings.kill_network_manager && !super::has_dependency("systemctl") {
        settings.kill_network_manager = false;
    }

    if Path::new(CONFIG_PATH).exists() {
        if let Ok(toml_settings) = toml::to_string(&settings) {
            std::fs::write(CONFIG_PATH, toml_settings).ok();

            log::debug!("settings saved into '{}'", CONFIG_PATH);
        }
    }
    *SETTINGS.lock().unwrap() = settings;
}

/// Get the current settings
pub fn get_settings() -> Settings {
    SETTINGS.lock().unwrap().clone()
}
