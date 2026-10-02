mod connections;
mod interfaces;
mod widgets;

use crate::backend;
use crate::types::Theme;
use interfaces::*;
use widgets::*;

use gtk4::prelude::*;
use gtk4::*;

use std::rc::Rc;
use std::sync::OnceLock;

/// The desktop's own GTK theme name and dark-theme preference, captured once before
/// we override them, so the "System" choice can restore exactly what we started with.
static ORIGINAL_THEME: OnceLock<(Option<String>, bool)> = OnceLock::new();

/// Apply a theme choice.
///
/// This is a plain GTK app (no libadwaita). GTK's `prefer-dark-theme` hint only has a
/// visible effect when the active theme ships a matching dark variant, which a custom
/// system theme often does not — so the explicit Light/Dark choices also pin the theme
/// to Adwaita, which is built into GTK and always honors the hint. "System" restores
/// whatever the desktop configured at startup.
pub fn apply_theme(theme: Theme) {
    let Some(settings) = gtk4::Settings::default() else {
        return;
    };

    let (original_theme, original_dark) = ORIGINAL_THEME.get_or_init(|| {
        (
            settings.gtk_theme_name().map(|name| name.to_string()),
            settings.property::<bool>("gtk-application-prefer-dark-theme"),
        )
    });

    let (theme_name, prefer_dark): (Option<&str>, bool) = match theme {
        Theme::System => (original_theme.as_deref(), *original_dark),
        Theme::Light => (Some("Adwaita"), false),
        Theme::Dark => (Some("Adwaita"), true),
    };

    settings.set_gtk_theme_name(theme_name);
    // Set via the property name rather than the typed setter, which is deprecated
    // since GTK 4.20.
    settings.set_property("gtk-application-prefer-dark-theme", prefer_dark);
}

pub fn build_ui(app: &Application) {
    // Settings must be loaded before the UI is built so the theme is applied from the
    // first frame.
    backend::load_settings();
    apply_theme(backend::get_settings().theme);

    let gui_data = Rc::new(AppData::new(app));

    connections::connect(app, gui_data.clone());

    gui_data.app_gui.show();

    if let Err(e) = backend::init() {
        PanicDialog::spawn(&gui_data.app_gui.window, &e.to_string())
    }
}

#[macro_export]
macro_rules! list_store_get {
    ($storage:expr,$iter:expr,$pos:expr,$typ:ty) => {
        $storage.get_value($iter, $pos).get::<$typ>().unwrap()
    };
}
