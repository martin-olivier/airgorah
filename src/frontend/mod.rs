mod app;
mod deauth;
mod dialog;
mod interface;
mod tools;

use crate::backend;
use crate::list_store_get;
use crate::types::*;
use dialog::*;
use tools::*;

use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use std::time::Duration;

use app::AppWindow;
use deauth::DeauthWindow;
use interface::InterfaceWindow;

pub fn app_setup(app: &Application) {
    backend::app_cleanup();

    ctrlc::set_handler(move || {
        backend::app_cleanup();
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    if sudo::check() != sudo::RunningAs::Root {
        return ErrorDialog::spawn(
            &app.active_window().unwrap(),
            "Error",
            "Airgorah need root privilege to run",
            true,
        );
    }

    if let Err(e) = backend::check_dependencies(&[
        "sh",
        "iw",
        "iwlist",
        "awk",
        "airmon-ng",
        "airodump-ng",
        "aireplay-ng",
    ]) {
        ErrorDialog::spawn(
            &app.active_window().unwrap(),
            "Missing dependencies",
            &e.to_string(),
            true,
        )
    }
}

pub fn build_ui(app: &Application) {
    let main_window = Rc::new(AppWindow::new(app));
    let interface_window = Rc::new(InterfaceWindow::new(app));

    app_setup(app);

    // Main window refresh

    let main_window_ref = main_window.clone();

    glib::timeout_add_local(Duration::from_millis(100), move || {
        match main_window_ref.aps_view.selection().selected() {
            Some((_, iter)) => {
                let bssid = list_store_get!(main_window_ref.aps_model, &iter, 1, String);
                let attack_pool = backend::get_attack_pool();

                match attack_pool.contains_key(&bssid) {
                    true => main_window_ref.deauth_but.set_label("Stop Attack"),
                    false => main_window_ref.deauth_but.set_label("Deauth Attack"),
                }
            }
            None => {
                main_window_ref.deauth_but.set_label("Deauth Attack");
            }
        };

        match backend::get_aps().is_empty() {
            true => main_window_ref.clear_but.set_sensitive(false),
            false => main_window_ref.clear_but.set_sensitive(true),
        }

        let aps = backend::get_airodump_data();

        for (bssid, ap) in aps.iter() {
            let it = match list_store_find(main_window_ref.aps_model.as_ref(), 1, bssid.as_str()) {
                Some(it) => it,
                None => main_window_ref.aps_model.append(),
            };

            let background_color = match backend::get_attack_pool().contains_key(bssid) {
                true => gdk::RGBA::RED,
                false => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
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
                    (10, &background_color.to_str()),
                ],
            );
        }

        if let Some((_, iter)) = main_window_ref.aps_view.selection().selected() {
            let bssid = list_store_get!(main_window_ref.aps_model, &iter, 1, String);
            let clients = &aps[&bssid].clients;

            for cli in clients.iter() {
                let it = match list_store_find(
                    main_window_ref.cli_model.as_ref(),
                    0,
                    cli.mac.as_str(),
                ) {
                    Some(it) => it,
                    None => main_window_ref.cli_model.append(),
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

                main_window_ref.cli_model.set(
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

    // Interfaces window callbacks

    let interface_window_ref = interface_window.clone();
    let scan_but = main_window.scan_but.clone();

    interface_window.select_but.connect_clicked(move |_| {
        let iter = match interface_window_ref.combo.active_iter() {
            Some(iter) => iter,
            None => return,
        };
        let iface = list_store_get!(interface_window_ref.model, &iter, 0, String);

        match crate::backend::enable_monitor_mode(&iface) {
            Ok(res) => {
                backend::set_iface(res);
                interface_window_ref.window.hide();
                scan_but.emit_clicked();
            }
            Err(e) => {
                ErrorDialog::spawn(
                    interface_window_ref.window.as_ref(),
                    "Monitor mode failed",
                    &format!("Could not enable monitor mode on \"{}\":\n{}", iface, e),
                    false,
                );
                interface_window_ref.refresh_but.emit_clicked();
            }
        };
    });

    // Scan button callback

    let run_scan = |main_window_ref: &AppWindow, interface_window: &InterfaceWindow| {
        let mut args = vec![];
        let channel_filter;

        let iface = match backend::get_iface() {
            Some(iface) => iface,
            None => return interface_window.window.show(),
        };

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
            if !backend::is_5ghz_supported(&iface).unwrap() {
                ErrorDialog::spawn(
                    main_window_ref.window.as_ref(),
                    "Error",
                    "Your network card doesn't support 5GHz",
                    false,
                );
                return main_window_ref.ghz_5_but.set_active(false);
            }
            bands.push('a');
        }
        if main_window_ref.ghz_2_4_but.is_active() {
            bands.push_str("bg");
        }
        args.push("--band");
        args.push(&bands);

        channel_filter = main_window_ref
            .channel_filter_entry
            .text()
            .as_str()
            .replace(" ", "");

        if !channel_filter.is_empty() {
            match backend::is_valid_channel_filter(&channel_filter) {
                true => {
                    args.push("--channel");
                    args.push(&channel_filter);
                }
                false => {
                    return ErrorDialog::spawn(
                        main_window_ref.window.as_ref(),
                        "Error",
                        "You need to put a valid channel filter",
                        false,
                    );
                }
            }
        }

        backend::set_scan_process(&args).unwrap_or_else(|e| {
            return ErrorDialog::spawn(
                main_window_ref.window.as_ref(),
                "Error",
                &format!("Could not start scan process: {}", e),
                false,
            );
        });

        main_window_ref.scan_but.set_icon_name("media-playback-pause-symbolic");
    };

    // Scan / Stop Callbacks

    let main_window_ref = main_window.clone();
    let interface_window_ref = interface_window.clone();

    main_window.scan_but.connect_clicked(move |this| {
        match backend::is_scan_process() {
            true => {
                backend::stop_scan_process();
                this.set_icon_name("media-playback-start-symbolic");
            }
            false => {
                run_scan(main_window_ref.as_ref(), interface_window_ref.as_ref());
            }
        }
    });

    let main_window_ref = main_window.clone();

    main_window.clear_but.connect_clicked(move |this| {
        backend::stop_scan_process();
        backend::get_aps().clear();

        main_window_ref.aps_model.clear();
        main_window_ref.cli_model.clear();

        this.set_sensitive(false);
        main_window_ref.scan_but.set_icon_name("media-playback-start-symbolic");
    });

    // 2.4 / 5 GHz buttons callback

    let main_window_ref = main_window.clone();
    let interface_window_ref = interface_window.clone();
    main_window.ghz_2_4_but.connect_toggled(move |_| {
        if backend::is_scan_process() {
            run_scan(main_window_ref.as_ref(), interface_window_ref.as_ref());
        }
    });

    let main_window_ref = main_window.clone();
    let interface_window_ref = interface_window.clone();
    main_window.ghz_5_but.connect_toggled(move |this| {
        let iface = match backend::get_iface() {
            Some(iface) => iface,
            None => return
        };

        if !backend::is_5ghz_supported(&iface).unwrap() {
            ErrorDialog::spawn(
                main_window_ref.window.as_ref(),
                "Error",
                "Your network card doesn't support 5GHz",
                false,
            );
            return this.set_active(false);
        }

        if backend::is_scan_process() {
            run_scan(main_window_ref.as_ref(), interface_window_ref.as_ref());
        }
    });

    let main_window_ref = main_window.clone();
    let interface_window_ref = interface_window.clone();
    main_window.channel_filter_entry.connect_text_notify(move |this| {
        let channel_filter = this
            .text()
            .as_str()
            .replace(" ", "");
    
        if !channel_filter.is_empty() && !backend::is_valid_channel_filter(&channel_filter) {
            return;
        }

        if backend::is_scan_process() {
            run_scan(main_window_ref.as_ref(), interface_window_ref.as_ref());
        }
    });

    // Deauth button callback

    let main_window_ref = main_window.clone();

    main_window.deauth_but.connect_clicked(move |_| {
        let (_model, iter) = match main_window_ref.aps_view.selection().selected() {
            Some((selection, iter)) => (selection, iter),
            None => return,
        };

        let bssid = list_store_get!(main_window_ref.aps_model, &iter, 1, String);
        let under_attack = backend::get_attack_pool().contains_key(&bssid);

        match under_attack {
            true => backend::stop_deauth_attack(&bssid),
            false => {
                if backend::is_scan_process() {
                    main_window_ref.scan_but.emit_clicked();
                }
                DeauthWindow::spawn(
                    main_window_ref.window.as_ref(),
                    backend::get_aps()[&bssid].clone(),
                );
            }
        }
    });
}
