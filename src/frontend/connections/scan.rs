use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::*;
use crate::list_store_get;

use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;

use chrono::Local;

fn run_scan(app_data: &AppData) {
    let iface = match backend::get_iface() {
        Some(iface) => iface,
        None => return app_data.interface_gui.show(),
    };

    let mut ghz_2_4 = false;
    let mut ghz_5 = false;

    if app_data.app_gui.ghz_5_but.is_active() {
        ghz_5 = true;
    }
    if app_data.app_gui.ghz_2_4_but.is_active() {
        ghz_2_4 = true;
    }

    let channel_filter = app_data.app_gui.channel_filter_entry.text();

    let channel_filter = match !channel_filter.is_empty() {
        true => match backend::is_valid_channel_filter(&channel_filter, ghz_2_4, ghz_5) {
            true => Some(channel_filter.to_string()),
            false => None,
        },
        false => None,
    };

    if let Err(e) = backend::set_scan_process(&iface, ghz_2_4, ghz_5, channel_filter) {
        return ErrorDialog::spawn(&app_data.app_gui.window, "Error", e.to_string().as_str());
    }

    app_data
        .app_gui
        .scan_but
        .set_icon_name("media-playback-pause-symbolic");
}

fn connect_scan_button(app_data: Rc<AppData>) {
    app_data.app_gui.scan_but.connect_clicked(
        clone!(@strong app_data => move |this| match backend::is_scan_process() {
            true => {
                backend::stop_scan_process().ok();
                this.set_icon_name("media-playback-start-symbolic");
            }
            false => {
                run_scan(&app_data);
            }
        }),
    );
}

fn connect_restart_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .restart_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            backend::stop_scan_process().ok();
            backend::get_aps().clear();
            backend::get_unlinked_clients().clear();

            app_data.app_gui.aps_model.clear();
            app_data.app_gui.cli_model.clear();

            run_scan(&app_data);
        }));
}

fn connect_export_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .export_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            if backend::get_aps().is_empty() {
                return;
            }

            let was_scanning = backend::is_scan_process();

            if was_scanning {
                app_data.app_gui.scan_but.emit_clicked();
            }

            let file_chooser_dialog = FileChooserDialog::new(
                Some("Save Capture"),
                Some(&app_data.app_gui.window),
                FileChooserAction::Save,
                &[("Save", ResponseType::Accept)],
            );

            let local = Local::now();
            let date = local.format("%Y-%m-%d-%Hh%M");

            file_chooser_dialog.set_current_name(&format!("capture_{date}.cap"));
            file_chooser_dialog.connect_close(
                clone!(@strong app_data => move |this| {
                    this.close();
                })
            );
            file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                if response == ResponseType::Accept {
                    this.close();

                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };
                    let path = gio_file.path().unwrap().to_str().unwrap().to_string();

                    if let Err(e) = backend::save_capture(&path) {
                        if was_scanning {
                            app_data.app_gui.scan_but.emit_clicked();
                        }
                        return ErrorDialog::spawn(&app_data.app_gui.window, "Save failed", &e.to_string());
                    }

                    for (_, ap) in backend::get_aps().iter_mut() {
                        if ap.handshake {
                            ap.saved_handshake = Some(path.to_owned());
                        }
                    }
                }

                if was_scanning {
                    app_data.app_gui.scan_but.emit_clicked();
                }
            }));
        }));
}

fn connect_report_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .report_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            if backend::get_aps().is_empty() {
                return;
            }

            let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                Some("Save capture report"),
                Some(&app_data.app_gui.window),
                FileChooserAction::Save,
                &[("Save", ResponseType::Accept)],
            ));

            let local = Local::now();
            let date = local.format("%Y-%m-%d-%Hh%M");

            file_chooser_dialog.set_current_name(&format!("report_{date}.json"));
            file_chooser_dialog.connect_close(
                clone!(@strong app_data => move |this| {
                    this.close();
                })
            );
            file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                if response == ResponseType::Accept {
                    this.close();

                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };

                    let path = gio_file.path().unwrap().to_str().unwrap().to_string();

                    if let Err(e) = backend::save_report(&path) {
                        return ErrorDialog::spawn(&app_data.app_gui.window, "Save failed", &e.to_string());
                    }
                }
            }));
        }));
}

fn connect_ghz_2_4_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .ghz_2_4_but
        .connect_toggled(clone!(@strong app_data => move |this| {
            if !this.is_active() && !app_data.app_gui.ghz_5_but.is_active() {
                if backend::is_scan_process() {
                    app_data.app_gui.scan_but.emit_clicked();
                }
                app_data.app_gui.scan_but.set_sensitive(false);
                app_data.app_gui.restart_but.set_sensitive(false);

                let filter = app_data.app_gui.channel_filter_entry.text();
                app_data.app_gui.channel_filter_entry.set_text(&filter);

                return;
            }

            run_scan(&app_data);

            app_data.app_gui.scan_but.set_sensitive(true);
            app_data.app_gui.restart_but.set_sensitive(true);

            let filter = app_data.app_gui.channel_filter_entry.text();
            app_data.app_gui.channel_filter_entry.set_text(&filter);
        }));
}

pub fn connect_ghz_5_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .ghz_5_but
        .connect_toggled(clone!(@strong app_data => move |this| {
            let iface = match backend::get_iface() {
                Some(iface) => iface,
                None => return,
            };

            if !backend::is_5ghz_supported(&iface).unwrap_or(false) && this.is_active() {
                ErrorDialog::spawn(
                    &app_data.app_gui.window,
                    "Error",
                    "Your network card doesn't support 5 GHz",
                );
                return this.set_active(false);
            }

            if !this.is_active() && !app_data.app_gui.ghz_2_4_but.is_active() {
                if backend::is_scan_process() {
                    app_data.app_gui.scan_but.emit_clicked();
                }
                app_data.app_gui.scan_but.set_sensitive(false);
                app_data.app_gui.restart_but.set_sensitive(false);

                let filter = app_data.app_gui.channel_filter_entry.text();
                app_data.app_gui.channel_filter_entry.set_text(&filter);

                return;
            }

            run_scan(&app_data);

            app_data.app_gui.scan_but.set_sensitive(true);
            app_data.app_gui.restart_but.set_sensitive(true);

            let filter = app_data.app_gui.channel_filter_entry.text();
            app_data.app_gui.channel_filter_entry.set_text(&filter);
        }));
}

fn connect_channel_entry(app_data: Rc<AppData>) {
    app_data.app_gui.channel_filter_entry.connect_text_notify(
        clone!(@strong app_data => move |this| {
            let channel_filter = this
                .text()
                .chars()
                .filter(|c| c.is_numeric() || *c == ',')
                .collect::<String>();

            if channel_filter != this.text() {
                return this.set_text(&channel_filter);
            }

            match channel_filter.is_empty() {
                true => app_data.app_gui.hopping_but.set_sensitive(false),
                false => app_data.app_gui.hopping_but.set_sensitive(true),
            }

            super::app::update_buttons_sensitivity(&app_data);

            let ghz_2_4_but = app_data.app_gui.ghz_2_4_but.is_active();
            let ghz_5_but = app_data.app_gui.ghz_5_but.is_active();

            if !channel_filter.is_empty() && !backend::is_valid_channel_filter(&channel_filter, ghz_2_4_but, ghz_5_but) {
                this.style_context().add_class("error");
            } else {
                this.style_context().remove_class("error");
            }

            if backend::is_scan_process() {
                run_scan(&app_data);
            }
        }),
    );
}

fn connect_cursor_changed(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .aps_view
        .connect_cursor_changed(clone!(@strong app_data => move |this| {
            super::app::update_buttons_sensitivity(&app_data);

            if let Some((_, it)) = this.selection().selected() {
                let essid = list_store_get!(app_data.app_gui.aps_model, &it, 0, String);
                let bssid = list_store_get!(app_data.app_gui.aps_model, &it, 1, String);
                let aps = backend::get_aps();

                app_data.app_gui.client_status_bar.pop(0);
                app_data.app_gui.client_status_bar.push(0, &format!("Showing '{essid}' clients"));

                let mut clients = match aps.get(&bssid) {
                    Some(ap) => ap.clients.keys().clone(),
                    None => return,
                };
                let mut cli_iter = app_data.app_gui.cli_model.iter_first();

                while let Some(it) = cli_iter {
                    let mac_val = list_store_get!(app_data.app_gui.cli_model, &it, 0, String);

                    if !clients.any(|x| &mac_val == x) {
                        break;
                    }

                    cli_iter = match app_data.app_gui.cli_model.iter_next(&it) {
                        true => Some(it),
                        false => return,
                    }
                }
            } else {
                app_data.app_gui.client_status_bar.pop(0);
                app_data.app_gui.client_status_bar.push(0, "Showing unassociated clients");
            }
            app_data.app_gui.cli_model.clear();
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_scan_button(app_data.clone());
    connect_restart_button(app_data.clone());
    connect_export_button(app_data.clone());
    connect_report_button(app_data.clone());

    connect_ghz_2_4_button(app_data.clone());
    connect_ghz_5_button(app_data.clone());

    connect_channel_entry(app_data.clone());
    connect_cursor_changed(app_data);
}
