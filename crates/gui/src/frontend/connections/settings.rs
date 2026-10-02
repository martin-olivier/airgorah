use crate::backend;
use crate::frontend::apply_theme;
use crate::frontend::interfaces::*;
use crate::types::Settings;

use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use regex::Regex;
use std::rc::Rc;

fn connect_controller(app_data: Rc<AppData>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(clone!(
        #[strong]
        app_data,
        move |_, key, _, _| {
            if key == gdk::Key::Escape {
                app_data.settings_gui.window.hide();
            }

            glib::Propagation::Proceed
        }
    ));

    app_data.settings_gui.window.add_controller(controller);
}

fn connect_random_mac_button(app_data: Rc<AppData>) {
    app_data.settings_gui.random_mac.connect_toggled(clone!(
        #[strong]
        app_data,
        move |_| {
            app_data.settings_gui.mac_entry.set_sensitive(false);
            app_data.settings_gui.save_but.set_sensitive(true);
        }
    ));
}

fn connect_default_mac_button(app_data: Rc<AppData>) {
    app_data.settings_gui.default_mac.connect_toggled(clone!(
        #[strong]
        app_data,
        move |_| {
            app_data.settings_gui.mac_entry.set_sensitive(false);
            app_data.settings_gui.save_but.set_sensitive(true);
        }
    ));
}

fn connect_specific_mac_button(app_data: Rc<AppData>) {
    app_data.settings_gui.specific_mac.connect_toggled(clone!(
        #[strong]
        app_data,
        move |_| {
            app_data.settings_gui.mac_entry.set_sensitive(true);
            app_data.settings_gui.mac_entry.notify("text");
        }
    ));
}

fn connect_mac_entry(app_data: Rc<AppData>) {
    app_data.settings_gui.mac_entry.connect_text_notify(
        clone!(#[strong] app_data, move |this| {
            let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})|([0-9a-fA-F]{4}\\.[0-9a-fA-F]{4}\\.[0-9a-fA-F]{4})$").unwrap();
            let entry = this.text().to_string();

            match mac_regex.is_match(&entry) {
                true => app_data.settings_gui.save_but.set_sensitive(true),
                false => app_data.settings_gui.save_but.set_sensitive(false),
            }
        }),
    );
}

/// Build a [`Settings`] from the current state of every settings widget.
fn collect_settings(app_data: &Rc<AppData>) -> Settings {
    let gui = &app_data.settings_gui;

    let mac_address = if gui.default_mac.is_active() {
        "default".to_string()
    } else if gui.specific_mac.is_active() {
        gui.mac_entry.text().to_string()
    } else {
        "random".to_string()
    };

    Settings {
        mac_address,
        display_hidden_ap: gui.display_hidden_ap.is_active(),
        kill_network_manager: gui.kill_network_manager.is_active(),
        hop_interval: gui.hop_interval.value() as u64,
        wordlist_path: gui.wordlist_entry.text().to_string(),
        save_path: gui.save_dir_entry.text().to_string(),
        theme: gui.selected_theme(),
        capture_notifications: gui.capture_notifications.is_active(),
    }
}

fn connect_save_but(app_data: Rc<AppData>) {
    app_data.settings_gui.save_but.connect_clicked(clone!(
        #[strong]
        app_data,
        move |_| {
            let settings = collect_settings(&app_data);

            backend::save_settings(settings.clone());

            // Capture notifications, the default paths and (on the next scan) the hop
            // interval are all re-read from settings where they are used. The theme
            // must be pushed to the live UI.
            apply_theme(settings.theme);

            app_data.settings_gui.window.hide();
        }
    ));
}

fn connect_reset_but(app_data: Rc<AppData>) {
    app_data.settings_gui.reset_but.connect_clicked(clone!(
        #[strong]
        app_data,
        move |_| {
            // Repopulate the widgets with the defaults; the user still clicks Save to
            // persist them.
            app_data.settings_gui.apply(&Settings::default());
            app_data.settings_gui.save_but.set_sensitive(true);
        }
    ));
}

fn connect_wordlist_but(app_data: Rc<AppData>) {
    app_data.settings_gui.wordlist_but.connect_clicked(clone!(
        #[strong]
        app_data,
        move |_| {
            let dialog = FileChooserDialog::new(
                Some("Select default wordlist"),
                Some(&app_data.settings_gui.window),
                FileChooserAction::Open,
                &[
                    ("Cancel", ResponseType::Cancel),
                    ("Open", ResponseType::Accept),
                ],
            );

            dialog.run_async(clone!(
                #[strong]
                app_data,
                move |this, response| {
                    this.close();

                    if response == ResponseType::Accept
                        && let Some(file) = this.file()
                        && let Some(path) = file.path()
                    {
                        app_data
                            .settings_gui
                            .wordlist_entry
                            .set_text(&path.to_string_lossy());
                    }
                }
            ));
        }
    ));
}

fn connect_save_dir_but(app_data: Rc<AppData>) {
    app_data.settings_gui.save_dir_but.connect_clicked(clone!(
        #[strong]
        app_data,
        move |_| {
            let dialog = FileChooserDialog::new(
                Some("Select default save directory"),
                Some(&app_data.settings_gui.window),
                FileChooserAction::SelectFolder,
                &[
                    ("Cancel", ResponseType::Cancel),
                    ("Select", ResponseType::Accept),
                ],
            );

            dialog.run_async(clone!(
                #[strong]
                app_data,
                move |this, response| {
                    this.close();

                    if response == ResponseType::Accept
                        && let Some(file) = this.file()
                        && let Some(path) = file.path()
                    {
                        app_data
                            .settings_gui
                            .save_dir_entry
                            .set_text(&path.to_string_lossy());
                    }
                }
            ));
        }
    ));
}

pub fn connect(app_data: Rc<AppData>) {
    if !backend::deps::is_installed(backend::deps::SYSTEMCTL) {
        app_data
            .settings_gui
            .kill_network_manager
            .set_sensitive(false);
        app_data
            .settings_gui
            .kill_network_manager
            .set_tooltip_text(Some("'systemd' is required to enable this option"));
    }

    connect_controller(app_data.clone());

    connect_random_mac_button(app_data.clone());
    connect_default_mac_button(app_data.clone());
    connect_specific_mac_button(app_data.clone());
    connect_mac_entry(app_data.clone());
    connect_wordlist_but(app_data.clone());
    connect_save_dir_but(app_data.clone());
    connect_reset_but(app_data.clone());
    connect_save_but(app_data);
}
