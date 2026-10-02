mod connections;
mod interfaces;
mod widgets;

use crate::backend;
use crate::types::{ColumnVisibility, Theme};
use interfaces::*;
use widgets::*;

use gtk4::prelude::*;
use gtk4::*;

use std::rc::Rc;
use std::sync::OnceLock;

/// The desktop's own dark-theme preference, captured once before we ever override
/// it, so the "System" choice can restore exactly what the user started with.
static ORIGINAL_PREFER_DARK: OnceLock<bool> = OnceLock::new();

/// Apply a theme choice through GTK's application-level dark-theme preference.
///
/// This is a plain GTK app (no libadwaita), so "System" means "leave the desktop
/// default untouched"; switching back to it restores the value captured at startup.
pub fn apply_theme(theme: Theme) {
    let Some(settings) = gtk4::Settings::default() else {
        return;
    };

    let original = *ORIGINAL_PREFER_DARK
        .get_or_init(|| settings.property::<bool>("gtk-application-prefer-dark-theme"));

    let prefer_dark = match theme {
        Theme::System => original,
        Theme::Light => false,
        Theme::Dark => true,
    };

    settings.set_property("gtk-application-prefer-dark-theme", prefer_dark);
}

/// Show or hide the toggleable access-point columns per the saved preferences.
/// ESSID (column 0) is the primary identifier and is always visible.
pub fn apply_column_visibility(view: &TreeView, columns: &ColumnVisibility) {
    let flags = [
        columns.bssid,
        columns.band,
        columns.channel,
        columns.power,
        columns.encryption,
        columns.clients,
        columns.first_time_seen,
        columns.last_time_seen,
        columns.handshake,
        columns.pmkid,
    ];

    let view_columns = view.columns();
    for (offset, visible) in flags.iter().enumerate() {
        if let Some(column) = view_columns.get(offset + 1) {
            column.set_visible(*visible);
        }
    }
}

pub fn build_ui(app: &Application) {
    // Settings must be loaded before the UI is built so the theme and column layout
    // are applied from the first frame.
    backend::load_settings();
    apply_theme(backend::get_settings().theme);

    let gui_data = Rc::new(AppData::new(app));

    apply_column_visibility(&gui_data.app_gui.aps_view, &backend::get_settings().columns);

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
