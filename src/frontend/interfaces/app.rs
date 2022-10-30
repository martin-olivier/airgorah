use crate::backend;
use crate::globals;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::*;

fn build_about_button() -> Button {
    let about_button = Button::builder().icon_name("dialog-information").build();
    about_button.connect_clicked(|_| {
        let icon = Pixbuf::from_read(std::io::BufReader::new(globals::APP_ICON)).unwrap();
        AboutDialog::builder()
            .program_name("Airgorah")
            .version(globals::VERSION)
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
    Button::builder()
        .icon_name("network-wireless-encrypted")
        .tooltip_text("Open the Handshake decryption pannel")
        .build()
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

fn build_interface_window(app: &Application) -> Window {
    Window::builder()
        .title("Select a wireless interface")
        .hide_on_close(true)
        .default_width(280)
        .default_height(70)
        .resizable(false)
        .modal(true)
        .transient_for(&app.active_window().unwrap())
        .build()
}

fn build_interface_model() -> ListStore {
    ListStore::new(&[glib::Type::STRING])
}

fn build_interface_view(model: &ListStore) -> ComboBox {
    let text_renderer = CellRendererText::new();

    let icon_renderer = CellRendererPixbuf::new();
    icon_renderer.set_property("icon-name", "network-wired");

    let combo = ComboBox::with_model(model);
    combo.set_hexpand(true);
    combo.pack_start(&icon_renderer, false);
    combo.pack_start(&text_renderer, false);
    combo.add_attribute(&text_renderer, "text", 0);

    combo
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

pub struct AppData {
    // Main window
    pub main_window: ApplicationWindow,
    pub aps_model: ListStore,
    pub aps_view: TreeView,
    pub cli_model: ListStore,
    pub cli_view: TreeView,
    pub ghz_2_4_but: CheckButton,
    pub ghz_5_but: CheckButton,
    pub channel_filter_entry: Entry,
    pub scan_but: Button,
    pub clear_but: Button,
    pub export_but: Button,
    pub deauth_but: Button,
    pub capture_but: Button,

    // Interface window
    pub interface_window: Window,
    pub select_but: Button,
    pub refresh_but: Button,
    pub interface_model: ListStore,
    pub interface_view: ComboBox,
}

impl AppData {
    pub fn new(app: &Application) -> Self {
        // --- MAIN WINDOW ---

        let main_window = build_main_window(app);
        main_window.set_titlebar(Some(&build_header_bar()));

        // Left Views (APs and Clients)

        let aps_model = build_aps_model();
        let aps_view = build_aps_view();
        let aps_scroll = build_aps_scroll();

        aps_scroll.set_child(Some(&aps_view));
        aps_view.set_model(Some(&aps_model));

        let cli_model = build_cli_model();
        let cli_view = build_cli_view();
        let cli_scroll = build_cli_scroll();

        cli_scroll.set_child(Some(&cli_view));
        cli_view.set_model(Some(&cli_model));

        // Scan, Stop and Save Buttons

        let scan_but = Button::builder()
            .icon_name("media-playback-start-symbolic")
            .tooltip_text("Start / Pause the scan")
            .build();

        let clear_but = Button::builder()
            .icon_name("edit-delete-symbolic")
            .sensitive(false)
            .tooltip_text("Stop the scan and clear scan results")
            .build();

        let export_but = Button::builder()
            .icon_name("media-floppy-symbolic")
            .tooltip_text("Export the scan results to a JSON file")
            .build();

        let top_but_box = CenterBox::new();
        top_but_box.set_margin_start(10);
        top_but_box.set_margin_end(10);
        top_but_box.set_margin_top(10);
        top_but_box.set_start_widget(Some(&scan_but));
        top_but_box.set_center_widget(Some(&clear_but));
        top_but_box.set_end_widget(Some(&export_but));

        // Scan filters

        let scan_box = Box::new(Orientation::Vertical, 10);

        let ghz_2_4_but = CheckButton::builder().active(true).label("2.4 GHz").build();
        let ghz_5_but = CheckButton::builder().active(false).label("5 GHz").build();

        ghz_2_4_but.set_margin_start(6);
        ghz_5_but.set_margin_end(6);

        let but_box = Box::new(Orientation::Horizontal, 10);
        but_box.append(&ghz_2_4_but);
        but_box.append(&ghz_5_but);
        but_box.set_margin_bottom(4);

        let band_frame = Frame::new(Some("Band"));
        band_frame.set_child(Some(&but_box));

        let channel_filter_entry = Entry::builder()
            .placeholder_text("ex: 1,6,11")
            .margin_start(4)
            .margin_end(4)
            .margin_bottom(4)
            .build();

        let channel_frame = Frame::new(Some("Channel"));
        channel_frame.set_child(Some(&channel_filter_entry));

        let separator = Separator::new(Orientation::Vertical);
        separator.set_vexpand(true);
        separator.set_opacity(0.0);

        let deauth_but = Button::builder()
            .label("Deauth Attack")
            .tooltip_text("Perform (or stop) a deauth attack on the selected AP")
            .sensitive(false)
            .build();

        let capture_but = Button::builder()
            .label("Handshake Capture")
            .tooltip_text("Capture a handshake from the selected AP")
            .sensitive(false)
            .margin_bottom(10)
            .build();

        scan_box.append(&band_frame);
        scan_box.append(&channel_frame);
        scan_box.append(&top_but_box);
        scan_box.append(&separator);
        scan_box.append(&deauth_but);
        scan_box.append(&capture_but);

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

        main_window.set_child(Some(&main_box));
        main_window.show();

        // --- INTERFACE WINDOW ---

        let interface_window = build_interface_window(app);

        let interface_model = build_interface_model();

        let interface_view = build_interface_view(&interface_model);

        let refresh_but = Button::builder()
            .icon_name("object-rotate-right-symbolic")
            .build();

        let select_but = Button::with_label("Select");

        let hbox = Box::new(Orientation::Horizontal, 10);
        let vbox = Box::new(Orientation::Vertical, 10);

        hbox.append(&interface_view);
        hbox.append(&refresh_but);

        hbox.set_margin_top(10);
        hbox.set_margin_start(10);
        hbox.set_margin_end(10);

        vbox.append(&hbox);
        vbox.append(&select_but);

        vbox.set_margin_top(0);

        interface_window.set_child(Some(&vbox));

        Self {
            // MAIN WINDOW
            main_window,
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
            capture_but,
            // INTERFACE WINDOW
            interface_window,
            select_but,
            refresh_but,
            interface_model,
            interface_view,
        }
    }
}
