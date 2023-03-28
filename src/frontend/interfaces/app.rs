use crate::backend;
use crate::frontend::widgets::*;
use crate::globals;

use gtk4::prelude::*;
use gtk4::*;

fn build_about_button() -> Button {
    Button::builder()
        .icon_name("dialog-information")
        .tooltip_text("About")
        .build()
}

fn build_update_button() -> Button {
    Button::builder()
        .icon_name("emblem-downloads")
        .tooltip_text("Update available")
        .build()
}

fn build_decrypt_button() -> Button {
    Button::builder()
        .icon_name("utilities-terminal")
        .tooltip_text("Open the handshake decryption panel")
        .build()
}

fn build_settings_button() -> Button {
    Button::builder()
        .icon_name("preferences-system")
        .tooltip_text("Open the settings panel")
        .build()
}

pub fn build_header_bar() -> HeaderBar {
    HeaderBar::builder().show_title_buttons(true).build()
}

fn build_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Airgorah")
        .default_width(1320)
        .default_height(620)
        .build();

    window.connect_close_request(|_| {
        backend::app_cleanup();
        glib::signal::Inhibit(false)
    });

    window
}

fn build_aps_model() -> ListStore {
    ListStore::new(&[
        glib::Type::STRING, // ESSID
        glib::Type::STRING, // BSSID
        glib::Type::STRING, // Band
        glib::Type::I32,    // Channel
        glib::Type::I32,    // Speed
        glib::Type::I32,    // Power
        glib::Type::STRING, // Encryption
        glib::Type::I32,    // Clients
        glib::Type::STRING, // First time seen
        glib::Type::STRING, // First time seen
        glib::Type::STRING, // Handshake
        glib::Type::STRING, // <color>
    ])
}

fn build_aps_view() -> TreeView {
    let view = TreeView::builder().vexpand(true).hexpand(true).build();
    let columns = [
        ("ESSID", 68),
        ("BSSID", 130),
        ("Band", 62),
        ("Channel", 80),
        ("Speed", 68),
        ("Power", 68),
        ("Encryption", 94),
        ("Clients", 72),
        ("First time seen", 140),
        ("Last time seen", 140),
        ("Handshake", 96),
    ];

    for (pos, (column_name, column_size)) in columns.into_iter().enumerate() {
        let column = TreeViewColumn::builder()
            .title(column_name)
            .resizable(false)
            .fixed_width(column_size)
            .sort_indicator(true)
            .sort_column_id(pos as i32)
            .expand(false)
            .build();

        if pos == 0 {
            let icon_renderer = CellRendererPixbuf::new();
            icon_renderer.set_property("icon-name", "network-wireless");

            column.pack_start(&icon_renderer, false);
            column.set_expand(true);
            column.set_min_width(column_size);
        }

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, false);
        column.add_attribute(&text_renderer, "text", pos as i32);
        column.add_attribute(&text_renderer, "background", 11);

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
        glib::Type::STRING, // Station MAC
        glib::Type::I32,    // Packets
        glib::Type::I32,    // Power
        glib::Type::STRING, // First time seen
        glib::Type::STRING, // Last time seen
        glib::Type::STRING, // <color>
    ])
}

fn build_cli_view() -> TreeView {
    let view = TreeView::builder().vexpand(true).hexpand(true).build();
    let column_names = [
        "Station MAC",
        "Packets",
        "Power",
        "First time seen",
        "Last time seen",
    ];

    for (pos, column_name) in column_names.into_iter().enumerate() {
        let column = TreeViewColumn::builder()
            .title(column_name)
            .resizable(false)
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

pub struct AppGui {
    // Header bar
    pub about_button: Button,
    pub update_button: Button,
    pub decrypt_button: Button,
    pub settings_button: Button,

    // Main window
    pub window: ApplicationWindow,
    pub aps_model: ListStore,
    pub aps_view: TreeView,
    pub cli_model: ListStore,
    pub cli_view: TreeView,
    pub iface_label: Label,
    pub ghz_2_4_but: CheckButton,
    pub ghz_5_but: CheckButton,
    pub channel_filter_entry: Entry,
    pub scan_but: Button,
    pub clear_but: Button,
    pub export_but: Button,
    pub deauth_but: IconTextButton,
    pub capture_but: IconTextButton,
}

impl AppGui {
    pub fn new(app: &Application) -> Self {
        let window = build_window(app);
        let header_bar = build_header_bar();

        window.set_titlebar(Some(&header_bar));

        let about_button = build_about_button();
        let update_button = build_update_button();
        let decrypt_button = build_decrypt_button();
        let settings_button = build_settings_button();

        header_bar.pack_start(&about_button);
        header_bar.pack_start(&decrypt_button);
        header_bar.pack_start(&settings_button);
        header_bar.pack_start(&update_button);

        update_button.hide();

        // Left View (Access Points and Clients)

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
            .tooltip_text("Stop the scan and clear results")
            .build();

        let export_but = Button::builder()
            .icon_name("media-floppy-symbolic")
            .tooltip_text("Save the capture")
            .build();

        let top_but_box = CenterBox::new();
        top_but_box.set_margin_start(10);
        top_but_box.set_margin_end(10);
        top_but_box.set_margin_top(10);
        top_but_box.set_start_widget(Some(&scan_but));
        top_but_box.set_center_widget(Some(&clear_but));
        top_but_box.set_end_widget(Some(&export_but));

        // Interface Display

        let iface_ico = Image::from_icon_name("network-wired");
        let iface_label = Label::new(Some("None"));

        let iface_box = Box::new(Orientation::Horizontal, 6);
        iface_box.append(&iface_ico);
        iface_box.append(&iface_label);

        iface_box.set_margin_top(4);
        iface_box.set_margin_start(6);
        iface_box.set_margin_end(6);
        iface_box.set_margin_bottom(4);

        let iface_frame = Frame::new(None);
        iface_frame.set_tooltip_text(Some("Wireless interface used for scans and attacks"));
        iface_frame.set_child(Some(&iface_box));

        // Scan filters

        let ghz_2_4_but = CheckButton::builder().active(true).label("2.4 GHz").build();
        let ghz_5_but = CheckButton::builder().active(false).label("5 GHz").build();

        let but_box = Box::new(Orientation::Horizontal, 10);
        but_box.append(&ghz_2_4_but);
        but_box.append(&ghz_5_but);

        but_box.set_margin_start(6);
        but_box.set_margin_end(6);
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

        let deauth_but = IconTextButton::new(globals::DEAUTH_ICON, "Deauth Attack");
        deauth_but.set_tooltip_text(Some(
            "Perform (or stop) a deauth attack on the selected access point",
        ));
        deauth_but.set_sensitive(false);

        let capture_but = IconTextButton::new(globals::CAPTURE_ICON, "Decrypt Handshake");
        capture_but.set_tooltip_text(Some(
            "Decrypt a handshake captured on the selected access point",
        ));
        capture_but.set_sensitive(false);
        capture_but.set_margin_bottom(10);

        let scan_box = Box::new(Orientation::Vertical, 10);
        scan_box.append(&iface_frame);
        scan_box.append(&band_frame);
        scan_box.append(&channel_frame);
        scan_box.append(&top_but_box);
        scan_box.append(&separator);
        scan_box.append(&deauth_but.handle);
        scan_box.append(&capture_but.handle);

        scan_box.set_hexpand(false);

        scan_box.set_margin_top(10);
        scan_box.set_margin_end(10);

        // Set main window child

        let panned = Paned::new(Orientation::Vertical);
        panned.set_wide_handle(true);
        panned.set_start_child(Some(&aps_scroll));
        panned.set_end_child(Some(&cli_scroll));

        let main_box = Box::new(Orientation::Horizontal, 10);
        main_box.append(&panned);
        main_box.append(&scan_box);

        window.set_child(Some(&main_box));
        window.show();

        Self {
            // Header bar
            about_button,
            update_button,
            decrypt_button,
            settings_button,
            // Main window
            window,
            aps_model,
            aps_view,
            cli_model,
            cli_view,
            iface_label,
            ghz_2_4_but,
            ghz_5_but,
            channel_filter_entry,
            scan_but,
            clear_but,
            export_but,
            deauth_but,
            capture_but,
        }
    }
}
