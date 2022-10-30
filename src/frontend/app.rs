use crate::backend;
use crate::frontend::dialog::*;
use crate::globals::*;
use crate::types::*;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::*;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

fn build_about_button() -> Button {
    let about_button = Button::builder().icon_name("dialog-information").build();
    about_button.connect_clicked(|_| {
        let icon = Pixbuf::from_read(std::io::BufReader::new(APP_ICON)).unwrap();
        AboutDialog::builder()
            .program_name("Airgorah")
            .version(VERSION)
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("Copyright (c) Martin OLIVIER")
            .license_type(License::MitX11)
            .logo(&Picture::for_pixbuf(&icon).paintable().unwrap())
            .comments("A WiFi auditing software that can performs deauth attacks and WPA passwords recovery")
            .website_label("https://github.com/martin-olivier/airgorah")
            .modal(true)
            .build()
            .show();
    });

    about_button
}

fn build_hs_decrypt_button() -> Button {
    let decrypt_hs_but = Button::builder()
        .icon_name("network-wireless-encrypted")
        .tooltip_text("Open the Handshake decryption pannel")
        .build();

    decrypt_hs_but
}

pub fn build_header_bar() -> HeaderBar {
    let header_bar = HeaderBar::builder().show_title_buttons(true).build();

    header_bar.pack_start(&build_about_button());
    header_bar.pack_end(&build_hs_decrypt_button());

    header_bar
}

fn build_main_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Airgorah")
        .default_width(1280)
        .default_height(540)
        .build();

    window.connect_close_request(|_| {
        backend::app_cleanup();
        glib::signal::Inhibit(false)
    });

    window
}

fn build_aps_model() -> ListStore {
    ListStore::new(&[
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
        glib::Type::STRING, // color
    ])
}

fn build_aps_view() -> TreeView {
    let view = TreeView::builder().vexpand(true).hexpand(true).build();
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

    for (pos, colomn_name) in colomn_names.into_iter().enumerate() {
        let column = TreeViewColumn::builder()
            .title(colomn_name)
            .resizable(true)
            .min_width(50)
            .sort_indicator(true)
            .sort_column_id(pos as i32)
            .expand(true)
            .build();

        if pos == 0 {
            let icon_renderer = CellRendererPixbuf::new();
            icon_renderer.set_property("icon-name", "network-wireless");
            column.pack_start(&icon_renderer, false);
        }

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos as i32);
        column.add_attribute(&text_renderer, "background", 10);

        view.append_column(&column);
    }
    view
}

fn build_aps_scroll() -> ScrolledWindow {
    let aps_scroll = ScrolledWindow::new();
    aps_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    aps_scroll.set_height_request(140);

    aps_scroll
}

fn build_cli_model() -> ListStore {
    ListStore::new(&[
        glib::Type::STRING,
        glib::Type::I32,
        glib::Type::I32,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
    ])
}

fn build_cli_view() -> TreeView {
    let view = TreeView::builder().vexpand(true).hexpand(true).build();
    let colomn_names = [
        "Station MAC",
        "Packets",
        "Power",
        "First time seen",
        "Last time seen",
    ];

    for (pos, colomn_name) in colomn_names.into_iter().enumerate() {
        let column = TreeViewColumn::builder()
            .title(colomn_name)
            .resizable(true)
            .min_width(50)
            .sort_indicator(true)
            .sort_column_id(pos as i32)
            .expand(true)
            .build();

        if pos == 0 {
            let icon_renderer = CellRendererPixbuf::new();
            icon_renderer.set_property("icon-name", "computer");
            column.pack_start(&icon_renderer, false);
        }

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos as i32);
        column.add_attribute(&text_renderer, "background", 5);

        view.append_column(&column);
    }
    view
}

fn build_cli_scroll() -> ScrolledWindow {
    let aps_scroll = ScrolledWindow::new();
    aps_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    aps_scroll
}

pub struct AppWindow {
    pub window: Rc<ApplicationWindow>,
    pub aps_model: Rc<ListStore>,
    pub aps_view: Rc<TreeView>,
    pub cli_model: Rc<ListStore>,
    pub cli_view: Rc<TreeView>,
    pub ghz_2_4_but: Rc<CheckButton>,
    pub ghz_5_but: Rc<CheckButton>,
    pub channel_filter_entry: Rc<Entry>,
    pub scan_but: Rc<Button>,
    pub clear_but: Rc<Button>,
    pub export_but: Rc<Button>,
    pub deauth_but: Rc<Button>,
}

impl AppWindow {
    pub fn new(app: &Application) -> Self {
        let window = Rc::new(build_main_window(app));
        window.set_titlebar(Some(&build_header_bar()));

        // Left Views (APs and Clients)

        let aps_model = Rc::new(build_aps_model());
        let aps_view = Rc::new(build_aps_view());
        let aps_scroll = build_aps_scroll();

        aps_scroll.set_child(Some(aps_view.as_ref()));
        aps_view.set_model(Some(aps_model.as_ref()));

        let cli_model = Rc::new(build_cli_model());
        let cli_view = Rc::new(build_cli_view());
        let cli_scroll = build_cli_scroll();

        cli_scroll.set_child(Some(cli_view.as_ref()));
        cli_view.set_model(Some(cli_model.as_ref()));

        // Scan, Stop and Save Buttons

        let scan_but = Rc::new(
            Button::builder()
                .icon_name("media-playback-start-symbolic")
                .tooltip_text("Start / Pause the scan")
                .build(),
        );

        let clear_but = Rc::new(
            Button::builder()
                .icon_name("edit-delete-symbolic")
                .sensitive(false)
                .tooltip_text("Stop the scan and clear scan results")
                .build(),
        );

        let export_but = Rc::new(
            Button::builder()
                .icon_name("media-floppy-symbolic")
                .tooltip_text("Export the scan results to a JSON file")
                .build(),
        );

        let top_but_box = CenterBox::new();
        top_but_box.set_margin_start(10);
        top_but_box.set_margin_end(10);
        top_but_box.set_margin_top(10);
        top_but_box.set_start_widget(Some(scan_but.as_ref()));
        top_but_box.set_center_widget(Some(clear_but.as_ref()));
        top_but_box.set_end_widget(Some(export_but.as_ref()));

        // Scan filters

        let scan_box = Box::new(Orientation::Vertical, 10);

        let ghz_2_4_but = Rc::new(CheckButton::builder().active(true).label("2.4 GHz").build());
        let ghz_5_but = Rc::new(CheckButton::builder().active(false).label("5 GHz").build());

        ghz_2_4_but.set_margin_start(6);
        ghz_5_but.set_margin_end(6);

        let but_box = Box::new(Orientation::Horizontal, 10);
        but_box.append(ghz_2_4_but.as_ref());
        but_box.append(ghz_5_but.as_ref());
        but_box.set_margin_bottom(4);

        let band_frame = Frame::new(Some("Band focus"));
        band_frame.set_child(Some(&but_box));

        let channel_filter_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: 1,6,11")
                .margin_start(4)
                .margin_end(4)
                .margin_bottom(4)
                .build(),
        );

        let channel_frame = Frame::new(Some("Channel focus"));
        channel_frame.set_child(Some(channel_filter_entry.as_ref()));

        let separator = Separator::new(Orientation::Vertical);
        separator.set_vexpand(true);
        separator.set_opacity(0.0);

        let deauth_but = Rc::new(
            Button::builder()
                .label("Deauth Attack")
                .tooltip_text("Perform (or stop) a deauth attack on the selected AP")
                .sensitive(false)
                .build(),
        );

        let capture_hs_but = Rc::new(
            Button::builder()
                .label("Handshake Capture")
                .tooltip_text("Capture a handshake from the selected AP")
                .sensitive(false)
                .margin_bottom(10)
                .build(),
        );

        scan_box.append(&band_frame);
        scan_box.append(&channel_frame);
        scan_box.append(&top_but_box);
        scan_box.append(&separator);
        scan_box.append(deauth_but.as_ref());
        scan_box.append(capture_hs_but.as_ref());

        scan_box.set_hexpand(false);

        scan_box.set_margin_top(10);
        scan_box.set_margin_end(10);

        // Set main window childs

        let main_box = Box::new(Orientation::Horizontal, 10);

        let panned = Paned::new(Orientation::Vertical);
        panned.set_wide_handle(true);
        panned.set_start_child(Some(&aps_scroll));
        panned.set_end_child(Some(&cli_scroll));

        main_box.append(&panned);
        main_box.append(&scan_box);

        window.set_child(Some(&main_box));
        window.show();

        // Set callbacks

        let window_ref = window.clone();

        export_but.connect_clicked(move |_| {
            let aps = APS.lock().unwrap();

            if aps.is_empty() {
                return ErrorDialog::spawn(
                    window_ref.as_ref(),
                    "Error",
                    "There is no data to export",
                    false,
                );
            }

            let json_data = serde_json::to_string::<Vec<AP>>(&aps.values().cloned().collect::<Vec<AP>>()).unwrap();

            let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                Some("Save Capture"),
                Some(window_ref.as_ref()),
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
        });

        let cli_model_ref = cli_model.clone();
        let deauth_but_ref = deauth_but.clone();
        let capture_hs_but_ref = capture_hs_but.clone();

        aps_view.connect_cursor_changed(move |this| {
            match this.selection().selected().is_some() {
                true => {
                    deauth_but_ref.set_sensitive(true);
                    capture_hs_but_ref.set_sensitive(true);
                }
                false => {
                    deauth_but_ref.set_sensitive(false);
                    capture_hs_but_ref.set_sensitive(false);
                }
            };
            cli_model_ref.clear();
        });

        let window_ref = window.clone();
        capture_hs_but.connect_clicked(move |_| {
            InfoDialog::spawn(
                window_ref.as_ref(),
                "Comming Soon",
                "This feature will be available in a future version",
            );
        });

        Self {
            window,
            aps_model,
            aps_view,
            cli_model,
            cli_view,
            ghz_2_4_but,
            ghz_5_but,
            channel_filter_entry,
            scan_but,
            clear_but,
            export_but,
            deauth_but,
        }
    }
}
