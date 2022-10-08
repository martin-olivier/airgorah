mod dialog;
mod interface;

use crate::backend;
use crate::globals::*;
use dialog::ErrorDialog;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::*;
use regex::Regex;
use std::rc::Rc;
use std::time::Duration;

fn build_aps_model() -> ListStore {
    let model = ListStore::new(&[
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::I32,
        glib::Type::I32,
        glib::Type::I32,
        glib::Type::STRING,
        glib::Type::I32,
        glib::Type::STRING,
        glib::Type::STRING,
    ]);

    model
}

fn build_aps_view() -> TreeView {
    let view = TreeView::new();
    let colomn_names = [
        "ESSID",
        "BSSID",
        "Band",
        "Channel",
        "Speed",
        "Power",
        "Encryption",
        "Clients",
        "First time seen",
        "Last time seen",
    ];
    let mut pos = 0;

    for colomn_name in colomn_names {
        let column = TreeViewColumn::builder()
            .title(colomn_name)
            .resizable(true)
            .min_width(50)
            .sort_indicator(true)
            .sort_column_id(pos)
            .expand(true)
            .build();

        /*let icon_renderer = CellRendererPixbuf::new();
        column.pack_start(&icon_renderer, false);
        column.add_attribute(&icon_renderer, "pixbuf", pos);*/

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos);

        view.append_column(&column);

        pos += 1;
    }
    view
    //renderer2.set_background(Some("Orange"));
}

//

fn build_cli_model() -> ListStore {
    let model = ListStore::new(&[
        glib::Type::STRING,
        glib::Type::I32,
        glib::Type::I32,
        glib::Type::STRING,
        glib::Type::STRING,
    ]);

    model
}

fn build_cli_view() -> TreeView {
    let view = TreeView::new();
    let colomn_names = [
        "Station MAC",
        "Packets",
        "Power",
        "First time seen",
        "Last time seen",
    ];
    let mut pos = 0;

    for colomn_name in colomn_names {
        let column = TreeViewColumn::builder()
            .title(colomn_name)
            .resizable(true)
            .min_width(50)
            .sort_indicator(true)
            .sort_column_id(pos)
            .expand(true)
            .build();

        /*let icon_renderer = CellRendererPixbuf::new();
        column.pack_start(&icon_renderer, false);
        column.add_attribute(&icon_renderer, "pixbuf", pos);*/

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos);

        view.append_column(&column);

        pos += 1;
    }
    view
}

pub fn build_ui(app: &Application) {
    std::fs::remove_file(SCAN_PATH.to_string() + "-01.csv").ok();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Airgorah")
        .default_width(1280)
        .default_height(540)
        .build();

    window.connect_close_request(|_| {
        if let Some(child) = SCAN_PROC.lock().unwrap().as_mut() {
            child.kill().unwrap();
            child.wait().unwrap();
        }
        if let Some(iface) = IFACE.lock().unwrap().as_ref() {
            backend::disable_monitor_mode(iface).ok();
        }
        glib::signal::Inhibit(false)
    });

    if sudo::check() != sudo::RunningAs::Root {
        return ErrorDialog::spawn(
            Some(&window),
            "Error",
            "Airgorah and its dependencies need root privilege to run",
            true,
        );
    }

    let aps_model = Rc::new(build_aps_model());
    let aps_view = build_aps_view();
    let aps_scroll = ScrolledWindow::new();

    aps_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    aps_scroll.set_height_request(140);
    aps_scroll.set_child(Some(&aps_view));

    aps_view.set_vexpand(true);
    aps_view.set_hexpand(true);
    aps_view.set_model(Some(&*aps_model));

    let cli_model = Rc::new(build_cli_model());
    let cli_view = build_cli_view();
    let cli_scroll = ScrolledWindow::new();

    cli_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    cli_scroll.set_child(Some(&cli_view));

    cli_view.set_vexpand(true);
    cli_view.set_hexpand(true);
    cli_view.set_model(Some(&*cli_model));

    let cli_model_ref = cli_model.clone();
    aps_view.connect_cursor_changed(move |_| {
        cli_model_ref.clear();
    });

    // SCAN

    let top_but_box = Box::new(Orientation::Horizontal, 10);

    let about_button = Button::new();
    about_button.set_icon_name("dialog-information");
    about_button.connect_clicked(|_| {
        let about = AboutDialog::builder()
            .program_name("Airgorah")
            .version("0.1 beta")
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("(c) Martin OLIVIER")
            .license_type(License::MitX11)
            .comments("A GUI around aircrack-ng suite tools")
            .logo_icon_name("network-wireless-hotspot")
            .website_label("https://github.com/martin-olivier/airgorah")
            .build();
        about.show();
    });

    let export_button = Button::new();
    export_button.set_icon_name("media-floppy");
    export_button.connect_clicked(|_| {
    });

    top_but_box.append(&about_button);
    top_but_box.append(&export_button);

    let scan_box = Box::new(Orientation::Vertical, 10);

    let ghz_2_4_but = CheckButton::builder().active(true).label("2.4 GHZ").build();
    let ghz_5_but = CheckButton::builder().active(false).label("5 GHZ").build();

    ghz_2_4_but.set_margin_start(6);
    ghz_5_but.set_margin_end(6);

    let but_box = Box::new(Orientation::Horizontal, 10);
    but_box.append(&ghz_2_4_but);
    but_box.append(&ghz_5_but);

    let band_frame = Frame::new(Some("Band"));
    band_frame.set_child(Some(&but_box));

    let channel_filter_entry = Rc::new(
        Entry::builder()
            .placeholder_text("ex: 1,6,11")
            .sensitive(false)
            .build(),
    );
    let channel_filter_but = CheckButton::builder().active(false).build();
    channel_filter_but.set_margin_start(10);

    let channel_filter_entry_ref = channel_filter_entry.clone();
    channel_filter_but.connect_toggled(move |this| {
        match this.is_active() {
            true => channel_filter_entry_ref.set_sensitive(true),
            false => channel_filter_entry_ref.set_sensitive(false),
        };
    });

    let bssid_filter_entry = Rc::new(
        Entry::builder()
            .placeholder_text("ex: 10:20:30:40:50")
            .sensitive(false)
            .build(),
    );
    let bssid_filter_but = CheckButton::builder().active(false).build();
    bssid_filter_but.set_margin_start(10);

    let bssid_filter_entry_other = bssid_filter_entry.clone();
    bssid_filter_but.connect_toggled(move |this| {
        match this.is_active() {
            true => bssid_filter_entry_other.set_sensitive(true),
            false => bssid_filter_entry_other.set_sensitive(false),
        };
    });

    let channel_box = Box::new(Orientation::Horizontal, 10);
    channel_box.append(&channel_filter_but);
    channel_box.append(channel_filter_entry.as_ref());

    let bssid_box = Box::new(Orientation::Horizontal, 10);
    bssid_box.append(&bssid_filter_but);
    bssid_box.append(bssid_filter_entry.as_ref());

    let channel_frame = Frame::new(Some("Channel filter"));
    channel_frame.set_child(Some(&channel_box));

    let bssid_frame = Frame::new(Some("BSSID filter"));
    bssid_frame.set_child(Some(&bssid_box));

    let scan_but = Button::with_label("Scan");

    let deauth_button = Button::with_label("Deauth attack");

    scan_box.append(&top_but_box);
    scan_box.append(&band_frame);
    scan_box.append(&channel_frame);
    scan_box.append(&bssid_frame);
    scan_box.append(&scan_but);
    scan_box.append(&deauth_button);

    scan_box.set_hexpand(false);

    scan_box.set_margin_top(10);
    scan_box.set_margin_end(10);

    //

    let main_box = Box::new(Orientation::Horizontal, 10);

    let panned = Paned::new(Orientation::Vertical);
    panned.set_wide_handle(true);
    panned.set_start_child(Some(&aps_scroll));
    panned.set_end_child(Some(&cli_scroll));

    main_box.append(&panned);
    main_box.append(&scan_box);

    window.set_child(Some(&main_box));
    window.show();

    // Refresh

    let aps_model_ref = aps_model.clone();
    let cli_model_ref = cli_model.clone();
    glib::timeout_add_local(Duration::from_millis(100), move || {
        match backend::get_airodump_data() {
            Some(aps) => {
                for ap in aps.iter() {
                    let mut iter = aps_model_ref.iter_first();
                    let mut already_there = false;

                    while let Some(it) = iter {
                        let val = aps_model_ref.get_value(&it, 1);
                        let bssid = val.get::<&str>().unwrap();

                        if bssid == ap.bssid {
                            aps_model_ref.set(
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
                            already_there = true;
                        }
                        iter = match aps_model_ref.iter_next(&it) {
                            true => Some(it),
                            false => None,
                        }
                    }

                    if already_there == false {
                        let it = aps_model_ref.append();
                        aps_model_ref.set(
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
                }
                match aps_view.selection().selected() {
                    Some(selection) => {
                        let val = aps_model_ref.get_value(&selection.1, 1);
                        let bssid = val.get::<&str>().unwrap();
                        let mut clients = vec![];

                        for ap in aps.iter() {
                            if ap.bssid == bssid {
                                clients = ap.clients.to_vec();
                                break;
                            }
                        }

                        for cli in clients.iter() {
                            let mut iter = cli_model_ref.iter_first();
                            let mut already_there = false;

                            while let Some(it) = iter {
                                let val = cli_model_ref.get_value(&it, 0);
                                let mac = val.get::<&str>().unwrap();

                                if mac == cli.mac {
                                    cli_model_ref.set(
                                        &it,
                                        &[
                                            (0, &cli.mac),
                                            (1, &cli.packets.parse::<i32>().unwrap_or(-1)),
                                            (2, &cli.power.parse::<i32>().unwrap_or(-1)),
                                            (3, &cli.first_time_seen),
                                            (4, &cli.last_time_seen),
                                        ],
                                    );
                                    already_there = true;
                                }
                                iter = match cli_model_ref.iter_next(&it) {
                                    true => Some(it),
                                    false => None,
                                }
                            }

                            if already_there == false {
                                let it = cli_model_ref.append();
                                cli_model_ref.set(
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
                    None => {}
                }
            }
            None => {}
        }
        glib::Continue(true)
    });

    // Actions

    let interface_window = interface::InterfaceWindow::new(&app);

    interface_window.select_but.connect_clicked(move |_| {
        let iter = match interface_window.combo.active_iter() {
            Some(iter) => iter,
            None => return,
        };
        let val = interface_window.model.get_value(&iter, 0);
        let iface = val.get::<&str>().unwrap();

        match crate::backend::enable_monitor_mode(iface) {
            Ok(res) => {
                IFACE.lock().unwrap().replace(res);
                interface_window.window.close();

                backend::set_scan_process(&vec![]);
            }
            Err(()) => {
                ErrorDialog::spawn(
                    Some(&interface_window.window),
                    "Monitor mode failed",
                    &format!("Could not enable monitor mode on \"{}\"", iface),
                    false,
                );
            }
        };
    });

    /*
        SCAN BUT
    */

    scan_but.connect_clicked(move |_| {
        let iface = match IFACE.lock().unwrap().as_ref() {
            Some(iface) => iface.to_string(),
            None => {
                return ErrorDialog::spawn(
                    Some(&window),
                    "Error",
                    "You need to select a network card first",
                    false,
                )
            }
        };
        let mut args = vec![];
        let mut channel_filter = "".to_string();
        let mut bssid_filter = "".to_string();

        if !ghz_2_4_but.is_active() && !ghz_5_but.is_active() {
            return ErrorDialog::spawn(
                Some(&window),
                "Error",
                "You need to select at least one frequency band",
                false,
            );
        }

        let mut bands = "".to_string();
        if ghz_5_but.is_active() {
            bands.push_str("a");
        }
        if ghz_2_4_but.is_active() {
            bands.push_str("bg");
        }
        args.push("--band");
        args.push(&bands);

        if channel_filter_but.is_active() {
            let channel_regex = Regex::new(r"^[1-9]+[0-9]*$").unwrap();
            let channel_list: Vec<String> = channel_filter_entry
                .text()
                .split_terminator(',')
                .map(String::from)
                .collect();
            for chan in channel_list {
                if !channel_regex.is_match(&chan) {
                    return ErrorDialog::spawn(
                        Some(&window),
                        "Error",
                        "You need to put a valid channel filter",
                        false,
                    );
                }
            }
            channel_filter = channel_filter_entry.text().to_string();
            args.push("--channel");
            args.push(&channel_filter);
        }

        if bssid_filter_but.is_active() {
            if !Regex::new(r"^([0-9a-fA-F][0-9a-fA-F]:){5}([0-9a-fA-F][0-9a-fA-F])$")
                .unwrap()
                .is_match(&bssid_filter_entry.text())
            {
                return ErrorDialog::spawn(
                    Some(&window),
                    "Error",
                    "You need to put a valid BSSID filter",
                    false,
                );
            }
            bssid_filter = bssid_filter_entry.text().to_string();
            args.push("--bssid");
            args.push(&bssid_filter);
        }
        backend::set_scan_process(&args);
        aps_model.clear();
        cli_model.clear();
    });
}
