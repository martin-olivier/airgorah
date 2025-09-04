use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::*;
use crate::globals;
use crate::list_store_get;
use crate::types::*;

use glib::ControlFlow;
use glib::clone;
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

fn get_channel_entries(entry: &Entry) -> Vec<i32> {
    let entry_text = String::from(entry.text().as_str());

    if entry_text.is_empty() {
        return Vec::new();
    }

    let channels: Vec<i32> = entry_text
        .split(',')
        .map(|num| num.parse::<i32>().unwrap())
        .collect();

    channels
}

fn connect_window_controller(app_data: Rc<AppData>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(clone!(@strong app_data => move |_, key, _, _| {
        if key == gdk::Key::Escape {
            app_data.app_gui.cli_view.selection().unselect_all();

            if app_data.app_gui.aps_view.selection().selected().is_some() {
                app_data.app_gui.aps_view.selection().unselect_all();
                app_data.app_gui.cli_model.clear();
            }

            app_data.app_gui.client_status_bar.pop(0);
            app_data.app_gui.client_status_bar.push(0, "Showing unassociated clients");

            update_buttons_sensitivity(&app_data);
        }

        glib::Propagation::Proceed
    }));

    app_data.app_gui.window.add_controller(controller);
}

fn connect_aps_controller(app: &Application, app_data: Rc<AppData>) {
    let gesture = GestureClick::new();
    gesture.set_button(gdk::ffi::GDK_BUTTON_SECONDARY as u32);
    gesture.connect_pressed(clone!(@strong app_data => move |gesture, _, x, y| {
        gesture.set_state(EventSequenceState::Claimed);

        if app_data.app_gui.aps_view.selection().selected().is_some() {
            let pos = gdk::Rectangle::new(x as i32, y as i32, 0, 0);

            app_data.app_gui.aps_menu.set_pointing_to(Some(&pos));
            app_data.app_gui.aps_menu.popup();
        }
    }));

    app_data.app_gui.aps_scroll.add_controller(gesture);

    let copy_bssid = gio::SimpleAction::new("copy_bssid", None);
    copy_bssid.connect_activate(clone!(@strong app_data => move |_, _| {
        let iter = match app_data.app_gui.aps_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);

        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&bssid);
        }
    }));
    app.add_action(&copy_bssid);

    let copy_essid = gio::SimpleAction::new("copy_essid", None);
    copy_essid.connect_activate(clone!(@strong app_data => move |_, _| {
        let iter = match app_data.app_gui.aps_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let essid = list_store_get!(app_data.app_gui.aps_model, &iter, 0, String);

        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&essid);
        }
    }));
    app.add_action(&copy_essid);

    let copy_channel = gio::SimpleAction::new("copy_channel", None);
    copy_channel.connect_activate(clone!(@strong app_data => move |_, _| {
        let iter = match app_data.app_gui.aps_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let channel = list_store_get!(app_data.app_gui.aps_model, &iter, 3, i32);

        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&channel.to_string());
        }
    }));
    app.add_action(&copy_channel);
}

fn connect_cli_controller(app: &Application, app_data: Rc<AppData>) {
    let gesture = GestureClick::new();
    gesture.set_button(gdk::ffi::GDK_BUTTON_SECONDARY as u32);
    gesture.connect_pressed(clone!(@strong app_data => move |gesture, _, x, y| {
        gesture.set_state(EventSequenceState::Claimed);

        if app_data.app_gui.cli_view.selection().selected().is_some() {
            let pos = gdk::Rectangle::new(x as i32, y as i32, 0, 0);

            app_data.app_gui.cli_menu.set_pointing_to(Some(&pos));
            app_data.app_gui.cli_menu.popup();
        }
    }));

    app_data.app_gui.cli_scroll.add_controller(gesture);

    let copy_mac = gio::SimpleAction::new("copy_mac", None);
    copy_mac.connect_activate(clone!(@strong app_data => move |_, _| {
        let iter = match app_data.app_gui.cli_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let mac = list_store_get!(app_data.app_gui.cli_model, &iter, 0, String);

        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&mac);
        }
    }));
    app.add_action(&copy_mac);

    let copy_vendor = gio::SimpleAction::new("copy_vendor", None);
    copy_vendor.connect_activate(clone!(@strong app_data => move |_, _| {
        let iter = match app_data.app_gui.cli_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let vendor = list_store_get!(app_data.app_gui.cli_model, &iter, 5, String);

        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&vendor);
        }
    }));
    app.add_action(&copy_vendor);

    let copy_probes = gio::SimpleAction::new("copy_probes", None);
    copy_probes.connect_activate(clone!(@strong app_data => move |_, _| {
        let iter = match app_data.app_gui.cli_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let probes = list_store_get!(app_data.app_gui.cli_model, &iter, 6, String);

        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(&probes);
        }
    }));
    app.add_action(&copy_probes);
}

fn connect_about_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .about_button
        .connect_clicked(clone!(@strong app_data => move |_| {
        let icon = Pixbuf::from_read(BufReader::new(globals::APP_ICON)).unwrap();
        let desc = "A WiFi security auditing software mainly based on aircrack-ng tools suite";

        AboutDialog::builder()
            .program_name("Airgorah")
            .version(globals::VERSION)
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("Copyright (c) Martin OLIVIER")
            .license_type(License::MitX11)
            .logo(&Picture::for_pixbuf(&icon).paintable().unwrap())
            .comments(desc)
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
        .connect_clicked(clone!(@strong app_data => move |this| {
            app_data.app_gui.channel_filter_entry.set_text("");

            this.set_sensitive(false);
            update_buttons_sensitivity(&app_data);
        }));
}

fn connect_focus_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .focus_but
        .connect_clicked(clone!(@strong app_data => move |this| {
            if let Some((_, iter)) = app_data.app_gui.aps_view.selection().selected() {
                let channel = list_store_get!(app_data.app_gui.aps_model, &iter, 3, i32);
                app_data.app_gui.channel_filter_entry.set_text(&channel.to_string());

                this.set_sensitive(false);
                app_data.app_gui.hopping_but.set_sensitive(true);
            }
        }));
}

fn connect_add_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .add_but
        .connect_clicked(clone!(@strong app_data => move |this| {
            if let Some((_, iter)) = app_data.app_gui.aps_view.selection().selected() {
                let channel = list_store_get!(app_data.app_gui.aps_model, &iter, 3, i32);
                let entry = app_data.app_gui.channel_filter_entry.text();
                let ghz_2_4_but = app_data.app_gui.ghz_2_4_but.is_active();
                let ghz_5_but = app_data.app_gui.ghz_5_but.is_active();

                if !backend::is_valid_channel_filter(&entry, ghz_2_4_but, ghz_5_but) {
                    return;
                }

                let entries = get_channel_entries(&app_data.app_gui.channel_filter_entry);

                if entries.contains(&channel) {
                    return;
                }

                let extend = match !entries.is_empty() {
                    true => format!(",{channel}"),
                    false => format!("{channel}")
                };

                if !backend::is_valid_channel_filter(&format!("{entry}{extend}"), ghz_2_4_but, ghz_5_but) {
                    return;
                }

                app_data.app_gui.channel_filter_entry.set_text(&format!("{entry}{extend}"));
                app_data.app_gui.hopping_but.set_sensitive(true);
                this.set_sensitive(false);
            }
        }));
}

pub fn update_buttons_sensitivity(app_data: &Rc<AppData>) {
    let iter = match app_data.app_gui.aps_view.selection().selected() {
        Some((_, iter)) => iter,
        None => {
            app_data.app_gui.focus_but.set_sensitive(false);
            app_data.app_gui.add_but.set_sensitive(false);
            app_data.app_gui.deauth_but.set_sensitive(false);
            app_data.app_gui.capture_but.set_sensitive(false);

            app_data.app_gui.previous_but.set_sensitive(false);
            app_data.app_gui.next_but.set_sensitive(false);

            match app_data.app_gui.aps_model.iter_first() {
                Some(_) => {
                    app_data.app_gui.top_but.set_sensitive(true);
                    app_data.app_gui.bottom_but.set_sensitive(true);
                }
                None => {
                    app_data.app_gui.top_but.set_sensitive(false);
                    app_data.app_gui.bottom_but.set_sensitive(false);
                }
            }

            return;
        }
    };

    let channel = list_store_get!(app_data.app_gui.aps_model, &iter, 3, i32);
    let entry = app_data.app_gui.channel_filter_entry.text();
    let ghz_2_4_but = app_data.app_gui.ghz_2_4_but.is_active();
    let ghz_5_but = app_data.app_gui.ghz_5_but.is_active();

    match channel
        != app_data
            .app_gui
            .channel_filter_entry
            .text()
            .parse::<i32>()
            .unwrap_or(-1)
        && backend::is_valid_channel_filter(&format!("{channel}"), ghz_2_4_but, ghz_5_but)
    {
        true => app_data.app_gui.focus_but.set_sensitive(true),
        false => app_data.app_gui.focus_but.set_sensitive(false),
    }

    match backend::is_valid_channel_filter(&entry, ghz_2_4_but, ghz_5_but) {
        true => {
            let entries = get_channel_entries(&app_data.app_gui.channel_filter_entry);
            match entries.contains(&channel) {
                true => app_data.app_gui.add_but.set_sensitive(false),
                false => {
                    let extand = match !entries.is_empty() {
                        true => format!(",{channel}"),
                        false => format!("{channel}"),
                    };
                    match backend::is_valid_channel_filter(
                        &format!("{entry}{extand}"),
                        ghz_2_4_but,
                        ghz_5_but,
                    ) {
                        true => app_data.app_gui.add_but.set_sensitive(true),
                        false => app_data.app_gui.add_but.set_sensitive(false),
                    }
                }
            }
        }
        false => app_data.app_gui.add_but.set_sensitive(false),
    }

    app_data.app_gui.deauth_but.set_sensitive(true);

    let prev_iter = iter;
    match app_data.app_gui.aps_model.iter_previous(&prev_iter) {
        true => {
            app_data.app_gui.previous_but.set_sensitive(true);
            app_data.app_gui.top_but.set_sensitive(true);
        }
        false => {
            app_data.app_gui.previous_but.set_sensitive(false);
            app_data.app_gui.top_but.set_sensitive(false);
        }
    }

    let next_iter = iter;
    match app_data.app_gui.aps_model.iter_next(&next_iter) {
        true => {
            app_data.app_gui.next_but.set_sensitive(true);
            app_data.app_gui.bottom_but.set_sensitive(true);
        }
        false => {
            app_data.app_gui.next_but.set_sensitive(false);
            app_data.app_gui.bottom_but.set_sensitive(false);
        }
    }
}

fn connect_previous_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .previous_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.app_gui.aps_view.selection().selected() {
                Some((_, iter)) => iter,
                None => return update_buttons_sensitivity(&app_data),
            };

            let prev_iter = iter;
            if !app_data.app_gui.aps_model.iter_previous(&prev_iter) {
                return update_buttons_sensitivity(&app_data);
            }

            let path = app_data.app_gui.aps_model.path(&prev_iter);
            app_data.app_gui.aps_view.selection().select_iter(&prev_iter);
            app_data.app_gui.aps_view.scroll_to_cell(Some(&path), None, false, 0.0, 0.0);
            app_data.app_gui.cli_model.clear();

            let essid = list_store_get!(app_data.app_gui.aps_model, &prev_iter, 0, String);
            app_data.app_gui.client_status_bar.pop(0);
            app_data.app_gui.client_status_bar.push(0, &format!("Showing '{essid}' clients"));

            update_buttons_sensitivity(&app_data);
        }));
}

fn connect_next_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .next_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.app_gui.aps_view.selection().selected() {
                Some((_, iter)) => iter,
                None => return update_buttons_sensitivity(&app_data),
            };

            let next_iter = iter;
            if !app_data.app_gui.aps_model.iter_next(&next_iter) {
                return update_buttons_sensitivity(&app_data);
            }

            let path = app_data.app_gui.aps_model.path(&next_iter);
            app_data.app_gui.aps_view.selection().select_iter(&next_iter);
            app_data.app_gui.aps_view.scroll_to_cell(Some(&path), None, false, 0.0, 0.0);
            app_data.app_gui.cli_model.clear();

            let essid = list_store_get!(app_data.app_gui.aps_model, &next_iter, 0, String);
            app_data.app_gui.client_status_bar.pop(0);
            app_data.app_gui.client_status_bar.push(0, &format!("Showing '{essid}' clients"));

            update_buttons_sensitivity(&app_data);
        }));
}

fn connect_top_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .top_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let first_iter = match app_data.app_gui.aps_model.iter_first() {
                Some(iter) => iter,
                None => return update_buttons_sensitivity(&app_data),
            };

            let path = app_data.app_gui.aps_model.path(&first_iter);
            app_data.app_gui.aps_view.selection().select_iter(&first_iter);
            app_data.app_gui.aps_view.scroll_to_cell(Some(&path), None, false, 0.0, 0.0);
            app_data.app_gui.cli_model.clear();

            let essid = list_store_get!(app_data.app_gui.aps_model, &first_iter, 0, String);
            app_data.app_gui.client_status_bar.pop(0);
            app_data.app_gui.client_status_bar.push(0, &format!("Showing '{essid}' clients"));

            update_buttons_sensitivity(&app_data);
        }));
}

fn connect_bottom_button(app_data: Rc<AppData>) {
    app_data
        .app_gui
        .bottom_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.app_gui.aps_model.iter_first() {
                Some(iter) => iter,
                None => return update_buttons_sensitivity(&app_data),
            };

            let mut last_iter = iter;

            while app_data.app_gui.aps_model.iter_next(&iter) {
                last_iter = iter;
            }

            let path = app_data.app_gui.aps_model.path(&last_iter);
            app_data.app_gui.aps_view.selection().select_iter(&last_iter);
            app_data.app_gui.aps_view.scroll_to_cell(Some(&path), None, false, 0.0, 0.0);
            app_data.app_gui.cli_model.clear();

            let essid = list_store_get!(app_data.app_gui.aps_model, &last_iter, 0, String);
            app_data.app_gui.client_status_bar.pop(0);
            app_data.app_gui.client_status_bar.push(0, &format!("Showing '{essid}' clients"));

            update_buttons_sensitivity(&app_data);
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
                        (10, &ap.handshake),
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
                            (5, &cli.vendor),
                            (6, &cli.probes),
                            (7, &background_color.to_str()),
                        ],
                    );

                    if app_data.deauth_gui.window.is_visible() && list_store_find(app_data.deauth_gui.store.as_ref(), 1, cli.mac.as_str()).is_none() {
                        app_data.deauth_gui.store.set(&app_data.deauth_gui.store.append(), &[(0, &false), (1, &cli.mac)]);
                    }
                }
            } else {
                let clients = backend::get_unlinked_clients().clone();

                for (_, cli) in clients {
                    let it = match list_store_find(app_data.app_gui.cli_model.as_ref(), 0, cli.mac.as_str()) {
                        Some(it) => it,
                        None => app_data.app_gui.cli_model.append(),
                    };

                    app_data.app_gui.cli_model.set(
                        &it,
                        &[
                            (0, &cli.mac),
                            (1, &cli.packets.parse::<i32>().unwrap_or(-1)),
                            (2, &cli.power.parse::<i32>().unwrap_or(-1)),
                            (3, &cli.first_time_seen),
                            (4, &cli.last_time_seen),
                            (5, &cli.vendor),
                            (6, &cli.probes),
                            (7, &gdk::RGBA::new(0.0, 0.0, 0.0, 0.0).to_str()),
                        ],
                    );
                }
            }

            if !backend::get_aps().is_empty() {
                app_data.app_gui.export_but.set_sensitive(true);
                app_data.app_gui.report_but.set_sensitive(true);
            }

            update_buttons_sensitivity(&app_data);

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
    std::thread::spawn(|| {
        loop {
            backend::update_handshakes().ok();

            std::thread::sleep(Duration::from_millis(1500));
        }
    });
}

fn start_vendor_refresh() {
    std::thread::spawn(|| {
        loop {
            backend::update_vendors();

            std::thread::sleep(Duration::from_millis(500));
        }
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

            let essid = list_store_get!(app_data.app_gui.aps_model, &iter, 0, String);
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
                Some("Save capture"),
                Some(&app_data.app_gui.window),
                FileChooserAction::Save,
                &[
                    ("Cancel", ResponseType::Cancel),
                    ("Save", ResponseType::Accept)
                ],
            );

            file_chooser_dialog.set_current_name(&format!("{essid}.cap"));
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

pub fn connect(app: &Application, app_data: Rc<AppData>) {
    connect_window_controller(app_data.clone());

    connect_aps_controller(app, app_data.clone());
    connect_cli_controller(app, app_data.clone());

    connect_about_button(app_data.clone());
    connect_update_button(app_data.clone());
    connect_decrypt_button(app_data.clone());
    connect_settings_button(app_data.clone());

    connect_previous_button(app_data.clone());
    connect_next_button(app_data.clone());
    connect_top_button(app_data.clone());
    connect_bottom_button(app_data.clone());

    start_app_refresh(app_data.clone());

    start_handshake_refresh();
    start_vendor_refresh();
    start_update_checker();

    connect_hopping_button(app_data.clone());
    connect_focus_button(app_data.clone());
    connect_add_button(app_data.clone());

    connect_deauth_button(app_data.clone());
    connect_capture_button(app_data);
}
