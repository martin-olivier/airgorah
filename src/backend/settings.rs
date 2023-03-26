use crate::error::Error;
use crate::globals::*;
use crate::types::*;

use std::path::Path;

pub fn load_settings() {
    if Path::new(CONFIG_PATH).exists() {
        let config = std::fs::read_to_string(CONFIG_PATH).unwrap_or_default();
        let settings: Settings = toml::from_str(&config).unwrap_or_default();
        *SETTINGS.lock().unwrap() = settings;
    }
}

pub fn save_settings(settings: Settings) {
    if Path::new(CONFIG_PATH).exists() {
        toml::to_string(&settings)
            .map_err(|_| Error::new("Error serializing settings"))
            .and_then(|config| {
                std::fs::write(CONFIG_PATH, config)
                    .map_err(|_| Error::new("Error writing settings to file"))
            })
            .ok();
    }
    *SETTINGS.lock().unwrap() = settings;
}

pub fn get_settings() -> Settings {
    SETTINGS.lock().unwrap().clone()
}