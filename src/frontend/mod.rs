mod connections;
mod interfaces;
mod widgets;

use crate::{backend, globals};
use connections::*;
use interfaces::*;

use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use glib::clone;
use std::time::Duration;

pub fn build_ui(app: &Application) {
    let gui_data = Rc::new(AppData::new(app));

    if let Err(e) = backend::app_setup() {
        return ErrorDialog::spawn(&app.active_window().unwrap(), "Error", &e.to_string(), true);
    }

    connect_about_button(gui_data.clone());
    connect_update_button(gui_data.clone());
    connect_decrypt_button(gui_data.clone());

    connect_app_refresh(gui_data.clone());
    connect_cursor_changed(gui_data.clone());

    connect_interface_refresh(gui_data.clone());
    connect_interface_select(gui_data.clone());

    connect_scan_button(gui_data.clone());
    connect_clear_button(gui_data.clone());
    connect_save_button(gui_data.clone());

    connect_ghz_2_4_button(gui_data.clone());
    connect_ghz_5_button(gui_data.clone());
    connect_channel_entry(gui_data.clone());

    connect_deauth_button(gui_data.clone());
    connect_capture_button(gui_data);
}

#[macro_export]
macro_rules! list_store_get {
    ($storage:expr,$iter:expr,$pos:expr,$typ:ty) => {
        $storage.get_value($iter, $pos).get::<$typ>().unwrap()
    };
}
