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
        .tooltip_text("Clear results and restart the scan")
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
        .sensitive(false)
        .build()
}

fn build_focus_button() -> Button {
    Button::builder()
        .icon_name("edit-select-symbolic")
        .tooltip_text("Focus the channel of the selected access point")
        .sensitive(false)
        .build()
}

fn build_add_button() -> Button {
    Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add the channel of the selected access point to the channel hop list")
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

    window.set_size_request(500, 500);
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
        glib::Type::BOOL,   // Handshake
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
            column.add_attribute(&icon_renderer, "cell-background", 11);
            column.set_expand(true);
        }

        if pos == 10 {
            let toggle = CellRendererToggle::new();
            toggle.set_sensitive(false);
            column.pack_start(&toggle, false);
            column.add_attribute(&toggle, "active", 10);
            column.add_attribute(&toggle, "cell-background", 11);
        } else {
            let text_renderer = CellRendererText::new();
            column.pack_start(&text_renderer, false);
            column.add_attribute(&text_renderer, "text", pos as i32);
            column.add_attribute(&text_renderer, "background", 11);
        }

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

fn build_aps_menu() -> PopoverMenu {
    let copy_bssid_item = gio::MenuItem::new(Some("Copy BSSID"), Some("app.copy_bssid"));
    let copy_essid_item = gio::MenuItem::new(Some("Copy ESSID"), Some("app.copy_essid"));
    let copy_channel_item = gio::MenuItem::new(Some("Copy Channel"), Some("app.copy_channel"));

    let submenu = gio::Menu::new();

    submenu.append_item(&copy_bssid_item);
    submenu.append_item(&copy_essid_item);
    submenu.append_item(&copy_channel_item);

    PopoverMenu::from_model(Some(&submenu))
}

fn build_cli_model() -> ListStore {
    ListStore::new(&[
        glib::Type::STRING, // Station MAC
        glib::Type::I32,    // Packets
        glib::Type::I32,    // Power
        glib::Type::STRING, // First time seen
        glib::Type::STRING, // Last time seen
        glib::Type::STRING, // Vendor
        glib::Type::STRING, // Probes
        glib::Type::STRING, // <color>
    ])
}

fn build_cli_view() -> TreeView {
    let view = TreeView::builder().vexpand(true).hexpand(true).build();
    let columns = [
        ("Station MAC", 200),
        ("Packets", 110),
        ("Power", 100),
        ("First time seen", 160),
        ("Last time seen", 160),
        ("Vendor", 200),
        ("Probes", 300),
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
            icon_renderer.set_property("icon-name", "computer");
            column.pack_start(&icon_renderer, false);
            column.add_attribute(&icon_renderer, "cell-background", 7);
        } else if pos == 5 || pos == 6 {
            column.set_expand(true);
        }

        let text_renderer = CellRendererText::new();
        column.pack_start(&text_renderer, true);
        column.add_attribute(&text_renderer, "text", pos as i32);
        column.add_attribute(&text_renderer, "background", 7);

        view.append_column(&column);
    }

    view
}

fn build_cli_scroll() -> ScrolledWindow {
    let aps_scroll = ScrolledWindow::new();
    aps_scroll.set_policy(PolicyType::Never, PolicyType::Automatic);

    aps_scroll
}

fn build_cli_menu() -> PopoverMenu {
    let copy_mac_item = gio::MenuItem::new(Some("Copy MAC"), Some("app.copy_mac"));
    let copy_vendor_item = gio::MenuItem::new(Some("Copy Vendor"), Some("app.copy_vendor"));
    let copy_probes_item = gio::MenuItem::new(Some("Copy Probes"), Some("app.copy_probes"));

    let submenu = gio::Menu::new();

    submenu.append_item(&copy_mac_item);
    submenu.append_item(&copy_vendor_item);
    submenu.append_item(&copy_probes_item);

    PopoverMenu::from_model(Some(&submenu))
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
    pub aps_scroll: ScrolledWindow,
    pub aps_menu: PopoverMenu,
    pub cli_model: ListStore,
    pub cli_view: TreeView,
    pub cli_scroll: ScrolledWindow,
    pub cli_menu: PopoverMenu,
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
    pub add_but: Button,
    pub deauth_but: IconButton,
    pub capture_but: IconButton,
    pub client_status_bar: Statusbar,
    pub iface_status_bar: Statusbar,
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
        let add_but = build_add_button();

        let previous_but = build_previous_but();
        let next_but = build_next_but();
        let top_but = build_top_but();
        let bottom_but = build_bottom_but();

        let about_button = build_about_button();
        let decrypt_button = build_decrypt_button();
        let settings_button = build_settings_button();
        let update_button = build_update_button();

        update_button.hide();

        // Scan filters

        let ghz_2_4_but = CheckButton::builder()
            .active(true)
            .sensitive(false)
            .label("2.4 GHz")
            .build();
        let ghz_5_but = CheckButton::builder()
            .active(false)
            .sensitive(false)
            .label("5 GHz")
            .build();

        // Channel

        let channel_filter_entry = Entry::builder()
            .placeholder_text("Channel (ex: 1,6,11)")
            .hexpand(true)
            .sensitive(false)
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

        header_bar.pack_end(&ghz_5_but);
        header_bar.pack_end(&ghz_2_4_but);

        header_bar.pack_end(&channel_filter_entry);

        header_bar.pack_end(&hopping_but);
        header_bar.pack_end(&focus_but);
        header_bar.pack_end(&add_but);

        // Left View (Access Points and Clients)

        let aps_model = build_aps_model();
        let aps_view = build_aps_view();
        let aps_scroll = build_aps_scroll();
        let aps_menu = build_aps_menu();

        aps_menu.set_parent(&aps_scroll);
        aps_scroll.set_child(Some(&aps_view));
        aps_view.set_model(Some(&aps_model));

        let cli_model = build_cli_model();
        let cli_view = build_cli_view();
        let cli_scroll = build_cli_scroll();
        let cli_menu = build_cli_menu();

        cli_menu.set_parent(&cli_scroll);
        cli_scroll.set_child(Some(&cli_view));
        cli_view.set_model(Some(&cli_model));

        // Set main window childs

        let panned_cli_aps = Paned::new(Orientation::Vertical);
        panned_cli_aps.set_wide_handle(true);
        panned_cli_aps.set_start_child(Some(&aps_scroll));
        panned_cli_aps.set_end_child(Some(&cli_scroll));

        let client_status_bar = Statusbar::new();
        client_status_bar.push(0, "Showing unassociated clients");

        let iface_status_bar = Statusbar::new();
        iface_status_bar.set_tooltip_text(Some("Wireless interface used for scans and attacks"));
        iface_status_bar.push(0, "No interface selected");

        let panned_status_bar = Paned::new(Orientation::Horizontal);
        panned_status_bar.set_wide_handle(true);
        panned_status_bar.set_start_child(Some(&client_status_bar));
        panned_status_bar.set_end_child(Some(&iface_status_bar));

        client_status_bar.set_size_request(110, -1);

        let vbox = Box::new(Orientation::Vertical, 0);
        vbox.append(&panned_cli_aps);
        vbox.append(&panned_status_bar);

        window.set_child(Some(&vbox));

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
            aps_scroll,
            aps_menu,
            cli_model,
            cli_view,
            cli_scroll,
            cli_menu,
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
            add_but,
            deauth_but,
            capture_but,
            client_status_bar,
            iface_status_bar,
        }
    }

    pub fn show(&self) {
        self.window.show();
    }
}
