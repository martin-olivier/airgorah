use crate::backend;
use crate::frontend::widgets::*;
use crate::globals;

use gtk4::prelude::*;
use gtk4::*;

fn build_about_button() -> Button {
    Button::builder()
        .icon_name("help-about-symbolic")
        .tooltip_text("About")
        .build()
}

fn build_update_button() -> Button {
    Button::builder()
        .icon_name("folder-download-symbolic")
        .tooltip_text("Update available")
        .build()
}

fn build_decrypt_button() -> Button {
    Button::builder()
        .icon_name("utilities-terminal-symbolic")
        .tooltip_text("Handshake decryption")
        .build()
}

fn build_settings_button() -> Button {
    Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text("Settings")
        .build()
}

fn build_scan_button() -> Button {
    Button::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text("Start / Pause the scan")
        .build()
}

fn build_restart_button() -> Button {
    Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Clear results and restarts the scan")
        .sensitive(false)
        .build()
}

fn build_export_button() -> Button {
    Button::builder()
        .icon_name("media-floppy-symbolic")
        .tooltip_text("Save captured packets as .cap file")
        .sensitive(false)
        .build()
}

fn build_report_button() -> Button {
    Button::builder()
        .icon_name("edit-paste-symbolic")
        .tooltip_text("Save captured data as .json file")
        .sensitive(false)
        .build()
}

fn build_hopping_button() -> Button {
    Button::builder()
        .icon_name("edit-select-all-symbolic")
        .tooltip_text("Hop on all channels of the selected bands")
        .build()
}

fn build_focus_button() -> Button {
    Button::builder()
        .icon_name("edit-select-symbolic")
        .tooltip_text("Focus the channel of the selected access point")
        .sensitive(false)
        .build()
}

fn build_previous_but() -> Button {
    Button::builder()
        .icon_name("go-up-symbolic")
        .tooltip_text("Previous access point")
        .sensitive(false)
        .build()
}

fn build_next_but() -> Button {
    Button::builder()
        .icon_name("go-down-symbolic")
        .tooltip_text("Next access point")
        .sensitive(false)
        .build()
}

fn build_top_but() -> Button {
    Button::builder()
        .icon_name("go-top-symbolic")
        .tooltip_text("First access point")
        .sensitive(false)
        .build()
}

fn build_bottom_but() -> Button {
    Button::builder()
        .icon_name("go-bottom-symbolic")
        .tooltip_text("Last access point")
        .sensitive(false)
        .build()
}

fn build_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("")
        .default_width(1240)
        .default_height(620)
        .build();

    window.connect_close_request(|_| {
        backend::app_cleanup();
        glib::Propagation::Proceed
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
        ("ESSID", 154),
        ("BSSID", 138),
        ("Band", 64),
        ("Channel", 86),
        ("Speed", 72),
        ("Power", 72),
        ("Encryption", 106),
        ("Clients", 80),
        ("First time seen", 150),
        ("Last time seen", 150),
        ("Handshake", 106),
    ];

    for (pos, (column_name, column_size)) in columns.into_iter().enumerate() {
        let column = TreeViewColumn::builder()
            .title(column_name)
            .resizable(true)
            .fixed_width(column_size)
            .min_width(column_size)
            .sort_indicator(true)
            .sort_column_id(pos as i32)
            .expand(false)
            .build();

        if pos == 0 {
            let icon_renderer = CellRendererPixbuf::new();
            icon_renderer.set_property("icon-name", "network-wireless");

            column.pack_start(&icon_renderer, false);
            column.set_expand(true);
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
    pub restart_but: Button,
    pub export_but: Button,
    pub report_but: Button,
    pub previous_but: Button,
    pub next_but: Button,
    pub top_but: Button,
    pub bottom_but: Button,
    pub hopping_but: Button,
    pub focus_but: Button,
    pub deauth_but: IconButton,
    pub capture_but: IconButton,
}

impl AppGui {
    pub fn new(app: &Application) -> Self {
        let window = build_window(app);
        let header_bar = HeaderBar::new();

        window.set_titlebar(Some(&header_bar));

        let scan_but = build_scan_button();
        let restart_but = build_restart_button();
        let export_but = build_export_button();
        let report_but = build_report_button();

        let hopping_but = build_hopping_button();
        let focus_but = build_focus_button();

        let previous_but = build_previous_but();
        let next_but = build_next_but();
        let top_but = build_top_but();
        let bottom_but = build_bottom_but();

        let about_button = build_about_button();
        let decrypt_button = build_decrypt_button();
        let settings_button = build_settings_button();
        let update_button = build_update_button();

        update_button.hide();

        // Interface Display

        let iface_ico = Image::from_icon_name("network-wired");
        let iface_label = Label::new(Some("None"));
        iface_label.set_tooltip_text(Some("Wireless interface used for scans and attacks"));

        // Scan filters

        let ghz_2_4_but = CheckButton::builder().active(true).label("2.4 GHz").build();
        let ghz_5_but = CheckButton::builder().active(false).label("5 GHz").build();

        // Channel

        let channel_filter_entry = Entry::builder()
            .placeholder_text("Channel (ex: 1,6,11)")
            .hexpand(true)
            .build();

        let css_provider = CssProvider::new();
        css_provider.load_from_data(
            std::str::from_utf8(
                b"
                    .error {
                        color: red;
                        border-color: red;
                    }
                ",
            )
            .unwrap(),
        );

        let style_context = channel_filter_entry.style_context();
        style_context.add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Actions

        let deauth_but = IconButton::new(globals::DEAUTH_ICON);
        deauth_but.set_tooltip_text(Some(
            "Perform (or stop) a deauth attack on the selected access point",
        ));
        deauth_but.set_sensitive(false);

        let capture_but = IconButton::new(globals::CAPTURE_ICON);
        capture_but.set_tooltip_text(Some(
            "Decrypt a handshake captured on the selected access point",
        ));
        capture_but.set_sensitive(false);

        // Header Bar

        header_bar.pack_start(&scan_but);
        header_bar.pack_start(&restart_but);
        header_bar.pack_start(&export_but);
        header_bar.pack_start(&report_but);

        header_bar.pack_start(&Separator::new(Orientation::Vertical));

        header_bar.pack_start(&previous_but);
        header_bar.pack_start(&next_but);
        header_bar.pack_start(&top_but);
        header_bar.pack_start(&bottom_but);

        header_bar.pack_start(&Separator::new(Orientation::Vertical));

        header_bar.pack_start(&deauth_but.handle);
        header_bar.pack_start(&capture_but.handle);

        header_bar.pack_start(&Separator::new(Orientation::Vertical));

        header_bar.pack_start(&decrypt_button);
        header_bar.pack_start(&settings_button);
        header_bar.pack_start(&about_button);
        header_bar.pack_start(&update_button);

        header_bar.pack_end(&iface_label);
        header_bar.pack_end(&iface_ico);

        header_bar.pack_end(&ghz_5_but);
        header_bar.pack_end(&ghz_2_4_but);

        header_bar.pack_end(&channel_filter_entry);

        header_bar.pack_end(&hopping_but);
        header_bar.pack_end(&focus_but);

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

        // Set main window child

        let panned = Paned::new(Orientation::Vertical);
        panned.set_wide_handle(true);
        panned.set_start_child(Some(&aps_scroll));
        panned.set_end_child(Some(&cli_scroll));

        window.set_child(Some(&panned));

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
            restart_but,
            export_but,
            report_but,
            previous_but,
            next_but,
            top_but,
            bottom_but,
            hopping_but,
            focus_but,
            deauth_but,
            capture_but,
        }
    }
}
