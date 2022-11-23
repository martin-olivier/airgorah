use crate::backend;
use crate::frontend::*;
use crate::frontend::interfaces::*;
use crate::types::*;
use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

fn run_scan(app_data: &AppData) {
    let mut args = vec![];

    let iface = match backend::get_iface() {
        Some(iface) => iface,
        None => return app_data.interface_gui.window.show(),
    };

    if !app_data.app_gui.ghz_2_4_but.is_active() && !app_data.app_gui.ghz_5_but.is_active() {
        return ErrorDialog::spawn(
            &app_data.app_gui.window,
            "Error",
            "You need to select at least one frequency band",
            false,
        );
    }

    let mut bands = "".to_string();

    if app_data.app_gui.ghz_5_but.is_active() {
        if !backend::is_5ghz_supported(&iface).unwrap() {
            ErrorDialog::spawn(
                &app_data.app_gui.window,
                "Error",
                "Your network card doesn't support 5GHz",
                false,
            );
            return app_data.app_gui.ghz_5_but.set_active(false);
        }
        bands.push('a');
    }
    if app_data.app_gui.ghz_2_4_but.is_active() {
        bands.push_str("bg");
    }
    args.push("--band");
    args.push(&bands);

    let channel_filter = app_data
        .app_gui.channel_filter_entry
        .text()
        .as_str()
        .replace(' ', "");

    if !channel_filter.is_empty() {
        match backend::is_valid_channel_filter(&channel_filter) {
            true => {
                args.push("--channel");
                args.push(&channel_filter);
            }
            false => {
                return ErrorDialog::spawn(
                    &app_data.app_gui.window,
                    "Error",
                    "You need to put a valid channel filter",
                    false,
                );
            }
        }
    }

    backend::set_scan_process(&args).unwrap_or_else(|e| {
        ErrorDialog::spawn(
            &app_data.app_gui.window,
            "Error",
            &format!("Could not start scan process: {}", e),
            false,
        )
    });

    app_data
        .app_gui
        .scan_but
        .set_icon_name("media-playback-pause-symbolic");
}

fn connect_scan_button(app_data: Rc<AppData>) {
    app_data.app_gui.scan_but.connect_clicked(
        clone!(@strong app_data => move |this| match backend::is_scan_process() {
            true => {
                backend::stop_scan_process();
                this.set_icon_name("media-playback-start-symbolic");
            }
            false => {
                run_scan(&app_data);
            }
        }),
    );
}

fn connect_clear_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .clear_but
        .connect_clicked(clone!(@strong app_data => move |this| {
            backend::stop_scan_process();
            backend::get_aps().clear();

            app_data.app_gui.aps_model.clear();
            app_data.app_gui.cli_model.clear();

            this.set_sensitive(false);
            app_data
                .app_gui
                .scan_but
                .set_icon_name("media-playback-start-symbolic");
        }));
}

fn connect_save_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .export_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let aps = backend::get_aps();
            if aps.is_empty() {
                return;
            }

            let aps = aps.values().cloned().collect::<Vec<AP>>();
            let json_data = serde_json::to_string::<Vec<AP>>(&aps).unwrap();

            let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                Some("Save Capture"),
                Some(&app_data.app_gui.window),
                FileChooserAction::Save,
                &[("Save", ResponseType::Accept)],
            ));

            file_chooser_dialog.set_current_name("capture.json");
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
            if backend::is_scan_process() {
                if !this.is_active() && !app_data.app_gui.ghz_5_but.is_active() {
                    ErrorDialog::spawn(
                        &app_data.app_gui.window,
                        "Error",
                        "You need to select at least one frequency band",
                        false,
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
            let iface = match backend::get_iface() {
                Some(iface) => iface,
                None => return,
            };

            if !backend::is_5ghz_supported(&iface).unwrap() && this.is_active() {
                ErrorDialog::spawn(
                    &app_data.app_gui.window,
                    "Error",
                    "Your network card doesn't support 5GHz",
                    false,
                );
                return this.set_active(false);
            }

            if backend::is_scan_process() {
                if !this.is_active() && !app_data.app_gui.ghz_2_4_but.is_active() {
                    ErrorDialog::spawn(
                        &app_data.app_gui.window,
                        "Error",
                        "You need to select at least one frequency band",
                        false,
                    );
                    return this.set_active(true);
                }
                run_scan(&app_data);
            }
        }));
}

fn connect_channel_entry(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .channel_filter_entry
        .connect_text_notify(clone!(@strong app_data => move |this| {
            let channel_filter = this
                .text()
                .as_str()
                .replace(' ', "");

            if !channel_filter.is_empty() && !backend::is_valid_channel_filter(&channel_filter) {
                return;
            }

            if backend::is_scan_process() {
                run_scan(&app_data);
            }
        }));
}

fn connect_cursor_changed(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .aps_view
        .connect_cursor_changed(clone!(@strong app_data => move |this| {
            match this.selection().selected().is_some() {
                true => {
                    app_data.app_gui.deauth_but.set_sensitive(true);
                    app_data.app_gui.capture_but.set_sensitive(true);
                }
                false => {
                    app_data.app_gui.deauth_but.set_sensitive(false);
                    app_data.app_gui.capture_but.set_sensitive(false);
                }
            };
            app_data.app_gui.cli_model.clear();
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_scan_button(app_data.clone());
    connect_clear_button(app_data.clone());
    connect_save_button(app_data.clone());

    connect_ghz_2_4_button(app_data.clone());
    connect_ghz_5_button(app_data.clone());

    connect_channel_entry(app_data.clone());
    connect_cursor_changed(app_data.clone());
}