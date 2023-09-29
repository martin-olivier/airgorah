use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::*;
use crate::types::*;

use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

fn run_scan(app_data: &AppData) {
    let iface = match backend::get_iface() {
        Some(iface) => iface,
        None => return app_data.interface_gui.window.show(),
    };

    if !app_data.app_gui.ghz_2_4_but.is_active() && !app_data.app_gui.ghz_5_but.is_active() {
        return ErrorDialog::spawn(
            &app_data.app_gui.window,
            "Error",
            "You need to select at least one frequency band",
        );
    }

    let mut ghz_2_4 = false;
    let mut ghz_5 = false;

    if app_data.app_gui.ghz_5_but.is_active() {
        if !backend::is_5ghz_supported(&iface).unwrap() {
            ErrorDialog::spawn(
                &app_data.app_gui.window,
                "Error",
                "Your network card doesn't support 5GHz",
            );
            return app_data.app_gui.ghz_5_but.set_active(false);
        }
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
        return ErrorDialog::spawn(
            &app_data.app_gui.window,
            "Error",
            &format!("Could not start scan process:\n\n{}", e),
        );
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
        .connect_clicked(clone!(@strong app_data => move |this| {
            backend::stop_scan_process().ok();
            backend::get_aps().clear();

            app_data.app_gui.aps_model.clear();
            app_data.app_gui.cli_model.clear();

            this.set_sensitive(false);

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

            file_chooser_dialog.set_current_name("capture.cap");
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

            let aps = backend::get_aps().values().cloned().collect::<Vec<AP>>();

            let json_data = serde_json::to_string::<Vec<AP>>(&aps).unwrap();

            let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                Some("Save capture report"),
                Some(&app_data.app_gui.window),
                FileChooserAction::Save,
                &[("Save", ResponseType::Accept)],
            ));

            file_chooser_dialog.set_current_name("report.json");
            file_chooser_dialog.run_async(move |this, response| {
                if response == ResponseType::Accept {
                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };
                    let mut file = File::create(gio_file.path().unwrap()).unwrap();
                    file.write_all(json_data.as_bytes()).unwrap();
                }
                this.close();
            });
        }));
}

fn connect_ghz_2_4_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .ghz_2_4_but
        .connect_toggled(clone!(@strong app_data => move |this| {
            let filter = app_data.app_gui.channel_filter_entry.text();
            app_data.app_gui.channel_filter_entry.set_text(&filter);

            if backend::is_scan_process() {
                if !this.is_active() && !app_data.app_gui.ghz_5_but.is_active() {
                    ErrorDialog::spawn(
                        &app_data.app_gui.window,
                        "Error",
                        "You need to select at least one frequency band",
                    );
                    return this.set_active(true);
                }
                run_scan(&app_data);
            }
        }));
}

pub fn connect_ghz_5_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .ghz_5_but
        .connect_toggled(clone!(@strong app_data => move |this| {
            let filter = app_data.app_gui.channel_filter_entry.text();
            app_data.app_gui.channel_filter_entry.set_text(&filter);

            let iface = match backend::get_iface() {
                Some(iface) => iface,
                None => return,
            };

            if !backend::is_5ghz_supported(&iface).unwrap_or(false) && this.is_active() {
                ErrorDialog::spawn(
                    &app_data.app_gui.window,
                    "Error",
                    "Your network card doesn't support 5GHz",
                );
                return this.set_active(false);
            }

            if backend::is_scan_process() {
                if !this.is_active() && !app_data.app_gui.ghz_2_4_but.is_active() {
                    ErrorDialog::spawn(
                        &app_data.app_gui.window,
                        "Error",
                        "You need to select at least one frequency band",
                    );
                    return this.set_active(true);
                }
                run_scan(&app_data);
            }
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
            match this.selection().selected().is_some() {
                true => {
                    app_data.app_gui.focus_but.set_sensitive(true);
                    app_data.app_gui.deauth_but.set_sensitive(true);
                }
                false => {
                    app_data.app_gui.focus_but.set_sensitive(false);
                    app_data.app_gui.deauth_but.set_sensitive(false);
                    app_data.app_gui.capture_but.set_sensitive(false);
                }
            };
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
