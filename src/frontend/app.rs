use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use crate::backend;
use crate::frontend::dialog::*;
use crate::globals::*;
use std::fs::File;
use std::io::prelude::*;

fn build_about_button() -> Button {
    let about_button = Button::builder().icon_name("dialog-information").build();
    about_button.connect_clicked(|_| {
        let about = AboutDialog::builder()
            .program_name("Airgorah")
            .version("0.1 beta")
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("Copyright (c) Martin OLIVIER")
            .license_type(License::MitX11)
            .comments("A GUI around aircrack-ng suite tools")
            .logo_icon_name("network-wireless-hotspot")
            .website_label("https://github.com/martin-olivier/airgorah")
            .build();
        about.show();
    });

    about_button
}

pub fn build_header_bar() -> HeaderBar {
    let header_bar = HeaderBar::builder().show_title_buttons(true).build();

    header_bar.pack_start(&build_about_button());

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
        glib::Type::STRING,
    ]);

    model
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

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos);
        column.add_attribute(&text_renderer, "background", 10);

        view.append_column(&column);

        pos += 1;
    }
    view
    //renderer2.set_background(Some("Orange"));
}

fn build_aps_scroll() -> ScrolledWindow {
    let aps_scroll = ScrolledWindow::new();
    aps_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    aps_scroll.set_height_request(140);

    aps_scroll
}

fn build_cli_model() -> ListStore {
    let model = ListStore::new(&[
        glib::Type::STRING,
        glib::Type::I32,
        glib::Type::I32,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
    ]);

    model
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

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos);
        column.add_attribute(&text_renderer, "background", 5);

        view.append_column(&column);

        pos += 1;
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
    pub channel_filter_but: Rc<CheckButton>,
    pub channel_filter_entry: Rc<Entry>,
    pub bssid_filter_but: Rc<CheckButton>,
    pub bssid_filter_entry: Rc<Entry>,
    pub scan_but: Rc<Button>,
    pub stop_but: Rc<Button>,
    pub export_but: Rc<Button>,
    pub deauth_but: Rc<Button>,
}

impl AppWindow {
    pub fn new(app: &Application) -> Self {
        let window = Rc::new(build_main_window(app));

        backend::app_cleanup();

        ctrlc::set_handler(move || {
            backend::app_cleanup();
            std::process::exit(1);
        })
        .expect("Error setting Ctrl-C handler");

        let header_bar = build_header_bar();
        window.set_titlebar(Some(&header_bar));

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

        let top_but_box = CenterBox::new();

        let scan_but = Rc::new(Button::builder().icon_name("media-playback-start").build());

        let stop_but = Rc::new(
            Button::builder()
                .icon_name("media-playback-stop")
                .sensitive(false)
                .build(),
        );

        let export_but = Rc::new(Button::builder().icon_name("media-floppy").build());

        top_but_box.set_margin_start(20);
        top_but_box.set_margin_end(20);
        top_but_box.set_margin_top(15);
        top_but_box.set_start_widget(Some(scan_but.as_ref()));
        top_but_box.set_center_widget(Some(stop_but.as_ref()));
        top_but_box.set_end_widget(Some(export_but.as_ref()));

        // Scan filters

        let scan_box = Box::new(Orientation::Vertical, 10);

        let ghz_2_4_but = Rc::new(CheckButton::builder().active(true).label("2.4 GHZ").build());
        let ghz_5_but = Rc::new(CheckButton::builder().active(true).label("5 GHZ").build());

        ghz_2_4_but.set_margin_start(6);
        ghz_5_but.set_margin_end(6);

        let but_box = Box::new(Orientation::Horizontal, 10);
        but_box.append(ghz_2_4_but.as_ref());
        but_box.append(ghz_5_but.as_ref());

        let band_frame = Frame::new(Some("Band"));
        band_frame.set_child(Some(&but_box));

        let channel_filter_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: 1,6,11")
                .sensitive(false)
                .build(),
        );
        let channel_filter_but = Rc::new(CheckButton::builder().active(false).build());
        channel_filter_but.set_margin_start(10);

        let bssid_filter_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: 10:20:30:40:50")
                .sensitive(false)
                .build(),
        );
        let bssid_filter_but = Rc::new(CheckButton::builder().active(false).build());
        bssid_filter_but.set_margin_start(10);

        let channel_box = Box::new(Orientation::Horizontal, 10);
        channel_box.append(channel_filter_but.as_ref());
        channel_box.append(channel_filter_entry.as_ref());

        let bssid_box = Box::new(Orientation::Horizontal, 10);
        bssid_box.append(bssid_filter_but.as_ref());
        bssid_box.append(bssid_filter_entry.as_ref());

        let channel_frame = Frame::new(Some("Channel filter"));
        channel_frame.set_child(Some(&channel_box));

        let bssid_frame = Frame::new(Some("BSSID filter"));
        bssid_frame.set_child(Some(&bssid_box));

        let separator = Separator::new(Orientation::Vertical);
        separator.set_vexpand(true);
        separator.set_opacity(0.0);

        let deauth_but = Rc::new(Button::with_label("Deauth Attack"));
        deauth_but.set_sensitive(false);

        let capture_hs_but = Rc::new(Button::with_label("Capture Handshake"));
        capture_hs_but.set_sensitive(false);

        let decrypt_hs_but = Rc::new(Button::with_label("Decrypt Handshake"));
        decrypt_hs_but.set_margin_bottom(10);

        scan_box.append(&band_frame);
        scan_box.append(&channel_frame);
        scan_box.append(&bssid_frame);
        scan_box.append(&top_but_box);
        scan_box.append(&separator);
        scan_box.append(deauth_but.as_ref());
        scan_box.append(capture_hs_but.as_ref());
        scan_box.append(decrypt_hs_but.as_ref());

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

        let scan_but_ref = scan_but.clone();

        stop_but.connect_clicked(move |but| {
            backend::stop_scan_process();
            but.set_sensitive(false);
            scan_but_ref.set_icon_name("media-playback-start");
        });

        let window_ref = window.clone();

        export_but.connect_clicked(move |_| {
            let aps = APS.lock().unwrap();

            if aps.is_empty() {
                return InfoDialog::spawn(window_ref.as_ref(), "Info", "There is no data to export");
            }

            let json_data = serde_json::to_string::<Vec<backend::AP>>(aps.as_ref()).unwrap();

            let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                Some("Save Capture"),
                Some(window_ref.as_ref()),
                FileChooserAction::Save,
                &[("Save", ResponseType::Accept)],
            ));

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

        let bssid_filter_entry_ref = bssid_filter_entry.clone();

        bssid_filter_but.connect_toggled(move |this| {
            match this.is_active() {
                true => bssid_filter_entry_ref.set_sensitive(true),
                false => bssid_filter_entry_ref.set_sensitive(false),
            };
        });

        let channel_filter_entry_ref = channel_filter_entry.clone();

        channel_filter_but.connect_toggled(move |this| {
            match this.is_active() {
                true => channel_filter_entry_ref.set_sensitive(true),
                false => channel_filter_entry_ref.set_sensitive(false),
            };
        });

        let window_ref = window.clone();
        capture_hs_but.connect_clicked(move |_| {
            InfoDialog::spawn(window_ref.as_ref(), "Comming Soon", "This feature will be available in a future version");
        });

        let window_ref = window.clone();
        decrypt_hs_but.connect_clicked(move |_| {
            InfoDialog::spawn(window_ref.as_ref(), "Comming Soon", "This feature will be available in a future version");
        });

        Self {
            window,
            aps_model,
            aps_view,
            cli_model,
            cli_view,
            ghz_2_4_but,
            ghz_5_but,
            channel_filter_but,
            channel_filter_entry,
            bssid_filter_but,
            bssid_filter_entry,
            scan_but,
            stop_but,
            export_but,
            deauth_but,
        }
    }
}
