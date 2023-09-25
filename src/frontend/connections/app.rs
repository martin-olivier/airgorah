use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::*;
use crate::globals;
use crate::list_store_get;
use crate::types::*;

use glib::clone;
use glib::ControlFlow;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::*;
use std::io::BufReader;
use std::rc::Rc;
use std::time::Duration;

fn list_store_find(storage: &ListStore, pos: i32, to_match: &str) -> Option<TreeIter> {
    let mut iter = storage.iter_first();

    while let Some(it) = iter {
        let value = list_store_get!(storage, &it, pos, String);

        if value == to_match {
            return Some(it);
        }

        iter = match storage.iter_next(&it) {
            true => Some(it),
            false => None,
        }
    }

    None
}

fn connect_about_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .about_button
        .connect_clicked(clone!(@strong app_data => move |_| {
        let ico = Pixbuf::from_read(BufReader::new(globals::APP_ICON)).unwrap();
        let des = "A WiFi auditing software that can perform deauth attacks and passwords cracking";

        AboutDialog::builder()
            .program_name("Airgorah")
            .version(globals::VERSION)
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("Copyright (c) Martin OLIVIER")
            .license_type(License::MitX11)
            .logo(&Picture::for_pixbuf(&ico).paintable().unwrap())
            .comments(des)
            .website_label("https://github.com/martin-olivier/airgorah")
            .transient_for(&app_data.app_gui.window)
            .modal(true)
            .build()
            .show();
        }));
}

fn connect_update_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .update_button
        .connect_clicked(clone!(@strong app_data => move |_| {
            let version = globals::VERSION;
            let new_version = globals::NEW_VERSION.lock().unwrap();

            let new_version = match new_version.as_ref() {
                Some(result) => result.clone(),
                None => "unknown".to_string(),
            };

            UpdateDialog::spawn(&app_data.app_gui.window, version, &new_version);
        }));
}

fn connect_decrypt_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .decrypt_button
        .connect_clicked(clone!(@strong app_data => move |_| {
            app_data.decrypt_gui.show(None);
        }));
}

fn connect_settings_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .settings_button
        .connect_clicked(clone!(@strong app_data => move |_| {
            app_data.settings_gui.show();
        }));
}

fn connect_hopping_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .hopping_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            app_data.app_gui.channel_filter_entry.set_text("");
        }));
}

fn connect_focus_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .focus_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            if let Some((_, iter)) = app_data.app_gui.aps_view.selection().selected() {
                let channel = list_store_get!(app_data.app_gui.aps_model, &iter, 3, i32);
                app_data.app_gui.channel_filter_entry.set_text(&channel.to_string());
            }
        }));
}

fn start_app_refresh(app_data: Rc<AppData>) {
    glib::timeout_add_local(
        Duration::from_millis(100),
        clone!(@strong app_data => move || {
            match app_data.app_gui.aps_view.selection().selected() {
                Some((_, iter)) => {
                    let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);
                    let attack_pool = backend::get_attack_pool();

                    match attack_pool.contains_key(&bssid) {
                        true => {
                            app_data.app_gui.deauth_but.set_icon(globals::STOP_ICON);
                        }
                        false => {
                            app_data.app_gui.deauth_but.set_icon(globals::DEAUTH_ICON);
                        }
                    }

                    match backend::get_aps()[&bssid].handshake {
                        true => app_data.app_gui.capture_but.set_sensitive(true),
                        false => app_data.app_gui.capture_but.set_sensitive(false),
                    }
                }
                None => {
                    app_data.app_gui.deauth_but.set_icon(globals::DEAUTH_ICON);
                }
            };

            match backend::get_aps().is_empty() {
                true => {
                    app_data.app_gui.restart_but.set_sensitive(false);
                    app_data.app_gui.export_but.set_sensitive(false);
                    app_data.app_gui.report_but.set_sensitive(false);
                    app_data.app_gui.top_but.set_sensitive(false);
                    app_data.app_gui.bottom_but.set_sensitive(false);
                    app_data.app_gui.previous_but.set_sensitive(false);
                    app_data.app_gui.next_but.set_sensitive(false);
                }
                false => {
                    app_data.app_gui.restart_but.set_sensitive(true);
                    app_data.app_gui.export_but.set_sensitive(true);
                    app_data.app_gui.report_but.set_sensitive(true);
                    app_data.app_gui.top_but.set_sensitive(true);
                    app_data.app_gui.bottom_but.set_sensitive(true);
                    app_data.app_gui.previous_but.set_sensitive(true);
                    app_data.app_gui.next_but.set_sensitive(true);
                }
            }

            let aps = backend::get_airodump_data();

            for (bssid, ap) in aps.iter() {
                if !backend::get_settings().display_hidden_ap && ap.hidden {
                    if let Some(iter) =
                        list_store_find(app_data.app_gui.aps_model.as_ref(), 1, bssid.as_str())
                    {
                        app_data.app_gui.aps_model.remove(&iter);
                    }
                    continue;
                }

                let it = match list_store_find(app_data.app_gui.aps_model.as_ref(), 1, bssid.as_str()) {
                    Some(it) => it,
                    None => app_data.app_gui.aps_model.append(),
                };

                let background_color = match backend::get_attack_pool().contains_key(bssid) {
                    true => gdk::RGBA::RED,
                    false => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
                };

                let handshake_status = match ap.handshake {
                    true => "Captured",
                    false => "",
                };

                app_data.app_gui.aps_model.set(
                    &it,
                    &[
                        (0, &ap.essid),
                        (1, &ap.bssid),
                        (2, &ap.band),
                        (3, &ap.channel.parse::<i32>().unwrap_or(-1)),
                        (4, &ap.speed.parse::<i32>().unwrap_or(-1)),
                        (5, &ap.power.parse::<i32>().unwrap_or(-1)),
                        (6, &ap.privacy),
                        (7, &(ap.clients.len() as i32)),
                        (8, &ap.first_time_seen),
                        (9, &ap.last_time_seen),
                        (10, &handshake_status),
                        (11, &background_color.to_str()),
                    ],
                );
            }

            if let Some((_, iter)) = app_data.app_gui.aps_view.selection().selected() {
                let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);
                let clients = &aps[&bssid].clients;

                for (_, cli) in clients.iter() {
                    let it =
                        match list_store_find(app_data.app_gui.cli_model.as_ref(), 0, cli.mac.as_str())
                        {
                            Some(it) => it,
                            None => app_data.app_gui.cli_model.append(),
                        };

                    let background_color = match backend::get_attack_pool().get(&bssid) {
                        Some((_, attack_target)) => match &attack_target {
                            AttackedClients::All(_) => gdk::RGBA::RED,
                            AttackedClients::Selection(selection) => {
                                let mut color = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);

                                for (sel, _) in selection.iter() {
                                    if sel == cli.mac.as_str() {
                                        color = gdk::RGBA::RED;
                                    }
                                }
                                color
                            }
                        },
                        None => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
                    };

                    app_data.app_gui.cli_model.set(
                        &it,
                        &[
                            (0, &cli.mac),
                            (1, &cli.packets.parse::<i32>().unwrap_or(-1)),
                            (2, &cli.power.parse::<i32>().unwrap_or(-1)),
                            (3, &cli.first_time_seen),
                            (4, &cli.last_time_seen),
                            (5, &background_color.to_str()),
                        ],
                    );
                }
            }

            ControlFlow::Continue
        }),
    );

    glib::timeout_add_local(
        Duration::from_millis(1000),
        clone!(@strong app_data => move || {
            let mut updater = globals::UPDATE_PROC.lock().unwrap();

            if let Some(proc) = updater.as_mut() {
                if proc.is_finished() {
                    if updater.take().unwrap().join().unwrap_or(false) {
                        app_data.app_gui.update_button.show();
                    }
                    return ControlFlow::Break;
                }
            }
            ControlFlow::Continue
        }),
    );
}

fn start_handshake_refresh() {
    std::thread::spawn(|| loop {
        backend::update_handshakes().ok();
        std::thread::sleep(Duration::from_millis(1500));
    });
}

fn start_update_checker() {
    globals::UPDATE_PROC
        .lock()
        .unwrap()
        .replace(std::thread::spawn(|| {
            let update = backend::check_update(globals::VERSION);

            match update {
                Some(update) => {
                    *globals::NEW_VERSION.lock().unwrap() = Some(update);
                    true
                }
                None => false,
            }
        }));
}

fn connect_deauth_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .deauth_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.app_gui.aps_view.selection().selected() {
                Some((_, iter)) => iter,
                None => return,
            };

            let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);
            let channel = list_store_get!(app_data.app_gui.aps_model, &iter, 3, i32);
            let under_attack = backend::get_attack_pool().contains_key(&bssid);

            match under_attack {
                true => backend::stop_deauth_attack(&bssid),
                false => {
                    app_data.app_gui.channel_filter_entry.set_text(&channel.to_string());
                    app_data.deauth_gui.show(backend::get_aps()[&bssid].clone());
                }
            }
        }));
}

fn connect_capture_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .capture_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.app_gui.aps_view.selection().selected() {
                Some((_, iter)) => iter,
                None => return,
            };

            let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);

            let ap = match backend::get_aps().get(&bssid) {
                Some(ap) => ap.clone(),
                None => return,
            };

            if !ap.handshake {
                return;
            }

            if let Some(ref cap) = ap.saved_handshake {
                return app_data.decrypt_gui.show(Some((cap.clone(), bssid)));
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
                        return ErrorDialog::spawn(&app_data.app_gui.window, "Save failed", &e.to_string());
                    }

                    for (_, ap) in backend::get_aps().iter_mut() {
                        if ap.handshake {
                            ap.saved_handshake = Some(path.to_owned());
                        }
                    }

                    app_data.decrypt_gui.show(Some((path, bssid)));
                }

                if was_scanning {
                    app_data.app_gui.scan_but.emit_clicked();
                }
            }));
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_about_button(app_data.clone());
    connect_update_button(app_data.clone());
    connect_decrypt_button(app_data.clone());
    connect_settings_button(app_data.clone());

    start_app_refresh(app_data.clone());
    start_handshake_refresh();
    start_update_checker();

    connect_hopping_button(app_data.clone());
    connect_focus_button(app_data.clone());
    connect_deauth_button(app_data.clone());
    connect_capture_button(app_data);
}
