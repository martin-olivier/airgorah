use crate::backend;
use crate::frontend::interfaces::*;
use crate::globals;
use crate::list_store_get;
use crate::types::*;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use std::time::Duration;

pub fn list_store_find(storage: &ListStore, pos: i32, to_match: &str) -> Option<TreeIter> {
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

pub fn connect_app_refresh(app_data: Rc<AppData>) {
    glib::timeout_add_local(Duration::from_millis(100), move || {
        match app_data.aps_view.selection().selected() {
            Some((_, iter)) => {
                let bssid = list_store_get!(app_data.aps_model, &iter, 1, String);
                let attack_pool = backend::get_attack_pool();

                match attack_pool.contains_key(&bssid) {
                    true => {
                        app_data.deauth_but.set_label("Stop Attack");
                        app_data.deauth_but.set_icon(globals::STOP_ICON);
                    }
                    false => {
                        app_data.deauth_but.set_label("Deauth Attack");
                        app_data.deauth_but.set_icon(globals::DEAUTH_ICON);
                    }
                }
            }
            None => {
                app_data.deauth_but.set_label("Deauth Attack");
                app_data.deauth_but.set_icon(globals::DEAUTH_ICON);
            }
        };

        match backend::get_aps().is_empty() {
            true => {
                app_data.clear_but.set_sensitive(false);
                app_data.export_but.set_sensitive(false);
            }
            false => {
                app_data.clear_but.set_sensitive(true);
                app_data.export_but.set_sensitive(true);
            }
        }

        let aps = backend::get_airodump_data();

        for (bssid, ap) in aps.iter() {
            let it = match list_store_find(app_data.aps_model.as_ref(), 1, bssid.as_str()) {
                Some(it) => it,
                None => app_data.aps_model.append(),
            };

            let background_color = match backend::get_attack_pool().contains_key(bssid) {
                true => gdk::RGBA::RED,
                false => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
            };

            app_data.aps_model.set(
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
                    (10, &background_color.to_str()),
                ],
            );
        }

        if let Some((_, iter)) = app_data.aps_view.selection().selected() {
            let bssid = list_store_get!(app_data.aps_model, &iter, 1, String);
            let clients = &aps[&bssid].clients;

            for (_, cli) in clients.iter() {
                let it = match list_store_find(app_data.cli_model.as_ref(), 0, cli.mac.as_str()) {
                    Some(it) => it,
                    None => app_data.cli_model.append(),
                };

                let background_color = match backend::get_attack_pool().get(&bssid) {
                    Some(attack_target) => match &attack_target.1 {
                        AttackedClients::All(_) => gdk::RGBA::RED,
                        AttackedClients::Selection(selection) => {
                            let mut color = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);

                            for sel in selection.iter() {
                                if sel.0 == cli.mac.as_str() {
                                    color = gdk::RGBA::RED;
                                }
                            }
                            color
                        }
                    },
                    None => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
                };

                app_data.cli_model.set(
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
        glib::Continue(true)
    });
}
