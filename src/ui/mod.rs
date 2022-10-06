mod dialog;
mod interface;
mod scan;

use crate::globals::IFACE;
use dialog::ErrorDialog;
use gtk4::prelude::*;
use gtk4::*;
use regex::Regex;
use std::rc::Rc;

fn build_aps_model() -> ListStore {
    let model = ListStore::new(&[
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
    ]);

    // fake values
    let it = model.append();
    model.set(
        &it,
        &[
            (0, &"NEUF_A598".to_string()),
            (1, &"10:20:30:40:50".to_string()),
            (2, &"2.4 GHz".to_string()),
            (3, &"6".to_string()),
            (4, &"55".to_string()),
            (5, &"95".to_string()),
            (6, &"WPA2".to_string()),
        ],
    );
    // fake values END
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
    ];
    let mut pos = 0;

    for colomn_name in colomn_names {
        let column = TreeViewColumn::builder()
            .title(colomn_name)
            .resizable(true)
            .min_width(50)
            .sort_indicator(true)
            .build();
        view.append_column(&column);

        let renderer = CellRendererText::new();
        column.pack_start(&renderer, true);
        column.add_attribute(&renderer, "text", pos);
        pos += 1;
    }
    view
    //renderer2.set_background(Some("Orange"));
}

//

fn build_cli_model() -> ListStore {
    let model = ListStore::new(&[glib::Type::STRING]);

    // fake values
    let it = model.append();
    model.set(&it, &[(0, &"01:21:32:45:22".to_string())]);
    // fake values END
    model
}

fn build_cli_view() -> TreeView {
    let view = TreeView::new();

    let column = TreeViewColumn::builder().title("Clients").build();
    view.append_column(&column);

    let renderer = CellRendererText::new();
    column.pack_start(&renderer, true);
    column.add_attribute(&renderer, "text", 0);

    view
    //renderer2.set_background(Some("Orange"));
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Airgorah")
        .default_width(850)
        .default_height(370)
        .build();

    let aps_model = build_aps_model();
    let aps_view = build_aps_view();

    aps_view.set_vexpand(true);
    aps_view.set_hexpand(true);
    aps_view.set_model(Some(&aps_model));

    let cli_model = build_cli_model();
    let cli_view = build_cli_view();

    cli_view.set_vexpand(true);
    cli_view.set_hexpand(true);
    cli_view.set_model(Some(&cli_model));

    // SCAN

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

    let scan_but = Button::with_label("Apply");

    let deauth_button = Button::with_label("Deauth attack");

    scan_box.append(&band_frame);
    scan_box.append(&channel_frame);
    scan_box.append(&bssid_frame);
    scan_box.append(&scan_but);
    scan_box.append(&deauth_button);

    scan_box.set_hexpand(false);

    scan_box.set_margin_top(10);
    scan_box.set_margin_end(10);

    //

    let about_button = Button::with_label("About");
    about_button.connect_clicked(|_| {
        let about = AboutDialog::new();
        about.show();
    });

    let main_box = Box::new(Orientation::Horizontal, 10);

    let panned = Paned::new(Orientation::Vertical);
    panned.set_wide_handle(true);
    panned.set_start_child(Some(&aps_view));
    panned.set_end_child(Some(&cli_view));

    main_box.append(&panned);
    main_box.append(&scan_box);

    window.set_child(Some(&main_box));
    window.show();

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
            Ok(str) => {
                IFACE.lock().unwrap().clear();
                IFACE.lock().unwrap().push_str(str.as_str());
                interface_window.window.close();
            }
            Err(()) => {
                ErrorDialog::spawn(
                    Some(&interface_window.window),
                    "Monitor mode failed",
                    &format!("Could not enable monitor mode on \"{}\"", iface),
                );
            }
        };
    });

    /*let scan_window_ref = scan_window.clone();
    scan_window.scan_but.connect_clicked(move |_| {
        // return if no IFACE
        if !scan_window_ref.ghz_2_4_but.is_active() && !scan_window_ref.ghz_5_but.is_active() {
            return dialog::ErrorDialog::spawn(
                Some(&scan_window_ref.window),
                "Error",
                "You need to select at least one frequency band",
            );
        }
        let chanel_filter = match scan_window_ref.channel_filter_but.is_active() {
            true => {
                if !Regex::new(r"^[1-9]+[0-9]*$")
                    .unwrap()
                    .is_match(&scan_window_ref.channel_filter_entry.text())
                {
                    return dialog::ErrorDialog::spawn(
                        Some(&scan_window_ref.window),
                        "Error",
                        "You need to put a valid channel filter",
                    );
                }
                "--channel ".to_string() + &scan_window_ref.channel_filter_entry.text()
            }
            false => "".to_string(),
        };
        //
        let bssid_filter = match scan_window_ref.bssid_filter_but.is_active() {
            true => {
                if !Regex::new(r"^([0-9a-fA-F][0-9a-fA-F]:){5}([0-9a-fA-F][0-9a-fA-F])$")
                    .unwrap()
                    .is_match(&scan_window_ref.bssid_filter_entry.text())
                {
                    return dialog::ErrorDialog::spawn(
                        Some(&scan_window_ref.window),
                        "Error",
                        "You need to put a valid BSSID filter",
                    );
                }
                "--bssid ".to_string() + &scan_window_ref.bssid_filter_entry.text()
            }
            false => "".to_string(),
        };
        let iface = IFACE.lock().unwrap();
        if iface.is_empty() {
            return dialog::ErrorDialog::spawn(
                Some(&scan_window_ref.window),
                "Error",
                "You need to select a network card first",
            );
        }
        let scan_duration = scan_window_ref.spin.value_as_int();
        //
        let mut child = std::process::Command::new("airodump-ng")
            .args([&iface, &chanel_filter, &bssid_filter])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("failed to execute process");
        //
        scan_window_ref.window.hide();
        let scan_window_ref2 = scan_window_ref.clone();
        let prog_win = progress::ProgressWindow::spawn(scan_duration as u64, move || {
            match child.try_wait() {
                Ok(Some(_exit_status)) => {
                    return dialog::ErrorDialog::spawn(
                        Some(&scan_window_ref2.window),
                        "Error",
                        "airodump-ng exited earlier",
                    );
                }
                Ok(None) => {
                    child.kill().unwrap();
                    let mut outbuf = vec![];
                    child
                        .stdout
                        .as_mut()
                        .unwrap()
                        .read_to_end(&mut outbuf)
                        .unwrap();
                    println!("{}", String::from_utf8_lossy(&outbuf));
                }
                Err(_) => {
                    return dialog::ErrorDialog::spawn(
                        Some(&scan_window_ref2.window),
                        "Error",
                        "fork error",
                    );
                }
            }
            println!("done");
        });
        prog_win.window.show();
    });*/

    // Note:
    /*
    faire un scan en continu /tmp/scan et lire le fichier grace à un glib::timer toute les 1 sec
    mettre les flitres de scan à droite
    Start/stop scan
    Clear
    */
}
