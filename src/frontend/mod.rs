mod app;
mod dialog;
mod interface;

use crate::backend;
use crate::globals::*;
use dialog::*;
use gtk4::prelude::*;
use gtk4::*;
use regex::Regex;
use std::rc::Rc;
use std::time::Duration;

use app::AppWindow;
use interface::InterfaceWindow;

pub fn build_ui(app: &Application) {

    let main_window = Rc::new(AppWindow::new(app));
    let interface_window = Rc::new(InterfaceWindow::new(app));

    if sudo::check() != sudo::RunningAs::Root {
        interface_window.window.hide();
        return ErrorDialog::spawn(
            main_window.window.as_ref(),
            "Error",
            "Airgorah and its dependencies need root privilege to run",
            true,
        );
    }

    // Main window refresh

    let main_window_ref = main_window.clone();

    glib::timeout_add_local(Duration::from_millis(100), move || {

        let list_store_find = |storage: &ListStore, pos: i32, to_match: &str| -> Option<TreeIter> {
            let mut iter = storage.iter_first();
        
            while let Some(it) = iter {
                let value = storage.get_value(&it, pos);
                let value_as_str = value.get::<&str>().unwrap();
        
                if value_as_str == to_match {
                    return Some(it);
                }
        
                iter = match storage.iter_next(&it) {
                    true => Some(it),
                    false => None,
                }
            }
        
            None
        };

        if let Some(aps) = backend::get_airodump_data() {
            for ap in aps.iter() {
                let it = match list_store_find(main_window_ref.aps_model.as_ref(), 1, &ap.bssid) {
                    Some(it) => it,
                    None => main_window_ref.aps_model.append(),
                };

                main_window_ref.aps_model.set(
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
                    ],
                );
            }
            if let Some(selection) = main_window_ref.aps_view.selection().selected() {
                let val = main_window_ref.aps_model.get_value(&selection.1, 1);
                let bssid = val.get::<&str>().unwrap();
                let mut clients = vec![];

                for ap in aps.iter() {
                    if ap.bssid == bssid {
                        clients = ap.clients.to_vec();
                        break;
                    }
                }

                for cli in clients.iter() {
                    let it = match list_store_find(main_window_ref.cli_model.as_ref(), 0, &cli.mac) {
                        Some(it) => it,
                        None => main_window_ref.cli_model.append(),
                    };

                    main_window_ref.cli_model.set(
                        &it,
                        &[
                            (0, &cli.mac),
                            (1, &cli.packets.parse::<i32>().unwrap_or(-1)),
                            (2, &cli.power.parse::<i32>().unwrap_or(-1)),
                            (3, &cli.first_time_seen),
                            (4, &cli.last_time_seen),
                        ],
                    );
                }
            }
        }
        glib::Continue(true)
    });

    // Interfaces window callbacks

    let interface_window_ref = interface_window.clone();
    let scan_but = main_window.scan_but.clone();

    interface_window.select_but.connect_clicked(move |_| {
        let iter = match interface_window_ref.combo.active_iter() {
            Some(iter) => iter,
            None => return,
        };
        let val = interface_window_ref.model.get_value(&iter, 0);
        let iface = val.get::<&str>().unwrap();

        match crate::backend::enable_monitor_mode(iface) {
            Ok(res) => {
                IFACE.lock().unwrap().replace(res);
                interface_window_ref.window.hide();

                scan_but.emit_clicked();
            }
            Err(()) => {
                ErrorDialog::spawn(
                    &interface_window_ref.window,
                    "Monitor mode failed",
                    &format!("Could not enable monitor mode on \"{}\"", iface),
                    false,
                );
            }
        };
    });

    // Main window callbacks

    let main_window_ref = main_window.clone();

    main_window.scan_but.connect_clicked(move |this| {
        let mut args = vec![];
        let channel_filter;
        let bssid_filter;

        if IFACE.lock().unwrap().is_none() {
            return interface_window.window.show();
        }

        if !main_window_ref.ghz_2_4_but.is_active() && !main_window_ref.ghz_5_but.is_active() {
            return ErrorDialog::spawn(
                main_window_ref.window.as_ref(),
                "Error",
                "You need to select at least one frequency band",
                false,
            );
        }

        let mut bands = "".to_string();
        if main_window_ref.ghz_5_but.is_active() {
            bands.push_str("a");
        }
        if main_window_ref.ghz_2_4_but.is_active() {
            bands.push_str("bg");
        }
        args.push("--band");
        args.push(&bands);

        if main_window_ref.channel_filter_but.is_active() {
            let channel_regex = Regex::new(r"^[1-9]+[0-9]*$").unwrap();
            let channel_list: Vec<String> = main_window_ref.channel_filter_entry
                .text()
                .split_terminator(',')
                .map(String::from)
                .collect();
            for chan in channel_list {
                if !channel_regex.is_match(&chan) {
                    return ErrorDialog::spawn(
                        main_window_ref.window.as_ref(),
                        "Error",
                        "You need to put a valid channel filter",
                        false,
                    );
                }
            }
            channel_filter = main_window_ref.channel_filter_entry.text().to_string();
            args.push("--channel");
            args.push(&channel_filter);
        }

        if main_window_ref.bssid_filter_but.is_active() {
            if !Regex::new(r"^([0-9a-fA-F][0-9a-fA-F]:){5}([0-9a-fA-F][0-9a-fA-F])$")
                .unwrap()
                .is_match(&main_window_ref.bssid_filter_entry.text())
            {
                return ErrorDialog::spawn(
                    main_window_ref.window.as_ref(),
                    "Error",
                    "You need to put a valid BSSID filter",
                    false,
                );
            }
            bssid_filter = main_window_ref.bssid_filter_entry.text().to_string();
            args.push("--bssid");
            args.push(&bssid_filter);
        }

        this.set_icon_name("object-rotate-right");
        main_window_ref.stop_but.set_sensitive(true);

        backend::launch_scan_process(&args);
        main_window_ref.aps_model.clear();
        main_window_ref.cli_model.clear();
    });
}
