use crate::globals::*;
use airgorah_common::deps;
use airgorah_common::types::Settings;

use std::path::PathBuf;

/// Per-user config path (`$XDG_CONFIG_HOME/airgorah/config.toml`, falling back to
/// `~/.config/...`).
fn user_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;

    Some(base.join("airgorah").join("config.toml"))
}

/// Load settings from the user config file, falling back to the built-in
/// defaults when it does not exist yet.
pub fn load_settings() {
    let mut settings = Settings::default();

    if let Some(path) = user_config_path()
        && let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(user) = toml::from_str::<Settings>(&content)
    {
        settings = user;
    }

    if settings.kill_network_manager && !deps::is_installed(deps::SYSTEMCTL) {
        settings.kill_network_manager = false;
    }

    log::debug!("settings loaded");

    *SETTINGS.lock().unwrap() = settings;
}

/// Save settings to the user config file.
pub fn save_settings(mut settings: Settings) {
    if settings.kill_network_manager && !deps::is_installed(deps::SYSTEMCTL) {
        settings.kill_network_manager = false;
    }

    if let Some(path) = user_config_path() {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).ok();
        }
        if let Ok(toml_settings) = toml::to_string(&settings) {
            std::fs::write(&path, toml_settings).ok();
            log::debug!("settings saved into '{}'", path.display());
        }
    }

    *SETTINGS.lock().unwrap() = settings;
}

/// Get the current settings.
pub fn get_settings() -> Settings {
    SETTINGS.lock().unwrap().clone()
}
