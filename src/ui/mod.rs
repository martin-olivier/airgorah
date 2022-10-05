mod dialog;
mod interface;
mod progress;
mod scan;

use crate::globals::IFACE;
use dialog::ErrorDialog;
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, Application, ApplicationWindow, Box, Button, CellRendererText, ListStore,
    Orientation, TreeView, TreeViewColumn,
};
use regex::Regex;
use std::io::Read;
use std::rc::Rc;

fn build_aps_view(view: &TreeView) {
    let colomn_names = [
        "ESSID",
        "BSSID",
        "Vendor",
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
    //renderer2.set_background(Some("Orange"));
}

pub fn build_ui(app: &Application) {
    sudo::escalate_if_needed().unwrap();

    let scan_window = Rc::new(scan::ScanWindow::new());

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Airgorah")
        .default_width(850)
        .default_height(370)
        .build();

    let main_box = Box::new(Orientation::Vertical, 10);
    let but_box = Box::new(Orientation::Horizontal, 10);

    let model = Rc::new(ListStore::new(&[
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
    ]));

    let it = model.append();
    model.set(
        &it,
        &[
            (0, &"NEUF_A598".to_string()),
            (1, &"10:20:30:40:50".to_string()),
            (2, &"APPLE INC".to_string()),
            (3, &"6".to_string()),
            (4, &"55".to_string()),
            (5, &"95".to_string()),
            (6, &"WPA2".to_string()),
        ],
    );

    let view = TreeView::new();
    build_aps_view(&view);

    view.set_vexpand(true);
    view.set_model(Some(&*model));

    let about_button = Button::with_label("About");
    about_button.connect_clicked(|_| {
        let about = AboutDialog::new();
        about.show();
    });

    let scan_button = Button::with_label("Scan");

    let scan_window_ref = scan_window.clone();
    scan_button.connect_clicked(move |_| {
        scan_window_ref.window.show();
    });

    main_box.append(&view);
    main_box.append(&but_box);

    but_box.append(&about_button);
    but_box.append(&scan_button);

    but_box.set_margin_end(10);
    but_box.set_margin_start(10);
    but_box.set_margin_bottom(10);

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

    let scan_window_ref = scan_window.clone();
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
    });
}
