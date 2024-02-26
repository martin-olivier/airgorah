use crate::backend;
use crate::frontend::interfaces::*;
use crate::types::Settings;

use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use regex::Regex;
use std::rc::Rc;

fn connect_controller(app_data: Rc<AppData>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(clone!(@strong app_data => move |_, key, _, _| {
        if key == gdk::Key::Escape {
            app_data.settings_gui.window.hide();
        }

        glib::Propagation::Proceed
    }));

    app_data.settings_gui.window.add_controller(controller);
}

fn connect_random_mac_button(app_data: Rc<AppData>) {
    app_data
        .settings_gui
        .random_mac
        .connect_toggled(clone!(@strong app_data => move |_| {
            app_data.settings_gui.mac_entry.set_sensitive(false);
            app_data.settings_gui.save_but.set_sensitive(true);
        }));
}

fn connect_default_mac_button(app_data: Rc<AppData>) {
    app_data
        .settings_gui
        .default_mac
        .connect_toggled(clone!(@strong app_data => move |_| {
            app_data.settings_gui.mac_entry.set_sensitive(false);
            app_data.settings_gui.save_but.set_sensitive(true);
        }));
}

fn connect_specific_mac_button(app_data: Rc<AppData>) {
    app_data
        .settings_gui
        .specific_mac
        .connect_toggled(clone!(@strong app_data => move |_| {
            app_data.settings_gui.mac_entry.set_sensitive(true);
            app_data.settings_gui.mac_entry.notify("text");
        }));
}

fn connect_mac_entry(app_data: Rc<AppData>) {
    app_data.settings_gui.mac_entry.connect_text_notify(
        clone!(@strong app_data => move |this| {
            let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})|([0-9a-fA-F]{4}\\.[0-9a-fA-F]{4}\\.[0-9a-fA-F]{4})$").unwrap();
            let entry = this.text().to_string();

            match mac_regex.is_match(&entry) {
                true => app_data.settings_gui.save_but.set_sensitive(true),
                false => app_data.settings_gui.save_but.set_sensitive(false),
            }
        }),
    );
}

fn connect_save_but(app_data: Rc<AppData>) {
    app_data
        .settings_gui
        .save_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let mut mac_address = Settings::default().mac_address;

            if app_data.settings_gui.random_mac.is_active() {
                mac_address = "random".to_string()
            } else if app_data.settings_gui.default_mac.is_active() {
                mac_address = "default".to_string()
            } else if app_data.settings_gui.specific_mac.is_active() {
                mac_address = app_data.settings_gui.mac_entry.text().to_string()
            }

            let display_hidden_ap = app_data.settings_gui.display_hidden_ap.is_active();
            let kill_network_manager = app_data.settings_gui.kill_network_manager.is_active();

            let settings = Settings {
                mac_address,
                display_hidden_ap,
                kill_network_manager,
            };

            backend::save_settings(settings);

            app_data.settings_gui.window.hide();
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    if !backend::has_dependency("systemctl") {
        app_data.settings_gui.kill_network_manager.set_sensitive(false);
        app_data.settings_gui.kill_network_manager.set_tooltip_text(
            Some("'systemd' is required to enable this option")
        );
    }

    connect_controller(app_data.clone());

    connect_random_mac_button(app_data.clone());
    connect_default_mac_button(app_data.clone());
    connect_specific_mac_button(app_data.clone());
    connect_mac_entry(app_data.clone());
    connect_save_but(app_data);
}
