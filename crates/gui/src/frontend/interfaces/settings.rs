use crate::backend;
use crate::types::{ColumnVisibility, Theme};

use gtk4::prelude::*;
use gtk4::*;

/// Toggleable access-point columns, in the model-column order that follows ESSID.
/// The order here must match [`ColumnVisibility`] field order below.
const COLUMN_LABELS: [&str; 10] = [
    "BSSID",
    "Band",
    "Channel",
    "Power",
    "Encryption",
    "Clients",
    "First time seen",
    "Last time seen",
    "Handshake",
    "PMKID",
];

/// Theme choices, in the order they appear in the drop-down.
const THEME_LABELS: [&str; 3] = ["System", "Light", "Dark"];

/// A content page with the app's standard margins.
fn page() -> Box {
    let vbox = Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);
    vbox
}

/// Wrap a page so it scrolls vertically when the window is small.
fn scrollable(child: &impl IsA<Widget>) -> ScrolledWindow {
    let scroll = ScrolledWindow::new();
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_child(Some(child));
    scroll
}

/// A labelled control laid out as `label … control`, wrapped in a titled frame.
fn labelled_row(title: &str, label: &str, control: &impl IsA<Widget>) -> Frame {
    let text = Label::new(Some(label));
    text.set_halign(Align::Start);
    text.set_hexpand(true);

    let row = Box::new(Orientation::Horizontal, 10);
    row.set_margin_start(4);
    row.set_margin_end(4);
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.append(&text);
    row.append(control);

    let frame = Frame::new(Some(title));
    frame.set_child(Some(&row));
    frame
}

pub struct SettingsGui {
    pub window: Window,

    // Interface
    pub random_mac: CheckButton,
    pub default_mac: CheckButton,
    pub specific_mac: CheckButton,
    pub mac_entry: Entry,
    pub kill_network_manager: CheckButton,

    // Scan
    pub hop_interval: SpinButton,
    pub refresh_rate: SpinButton,

    // Cracking
    pub wordlist_entry: Entry,
    pub wordlist_but: Button,
    pub save_dir_entry: Entry,
    pub save_dir_but: Button,

    // Display
    pub theme: DropDown,
    pub capture_notifications: CheckButton,
    pub display_hidden_ap: CheckButton,
    pub columns: Vec<CheckButton>,

    // Footer
    pub reset_but: Button,
    pub save_but: Button,
}

impl SettingsGui {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = Window::builder()
            .title("Settings")
            .hide_on_close(true)
            .default_width(430)
            .default_height(540)
            .resizable(true)
            .transient_for(parent)
            .modal(true)
            .build();

        // --- Interface tab -------------------------------------------------
        let random_mac = CheckButton::with_label("Random MAC address");
        let default_mac = CheckButton::with_label("Default MAC address");
        let specific_mac = CheckButton::with_label("Specific MAC address");
        let mac_entry = Entry::builder()
            .placeholder_text("ex: 00:00:01:02:03:04")
            .hexpand(true)
            .editable(true)
            .sensitive(false)
            .build();

        random_mac.set_active(true);
        default_mac.set_group(Some(&random_mac));
        specific_mac.set_group(Some(&random_mac));

        let mac_box = Box::new(Orientation::Vertical, 4);
        mac_box.set_margin_start(4);
        mac_box.set_margin_end(4);
        mac_box.set_margin_bottom(4);
        mac_box.append(&random_mac);
        mac_box.append(&default_mac);
        mac_box.append(&specific_mac);
        mac_box.append(&mac_entry);

        let mac_frame = Frame::new(Some("MAC"));
        mac_frame.set_child(Some(&mac_box));

        let kill_network_manager = CheckButton::with_label("Kill network managers");
        kill_network_manager.set_active(true);
        kill_network_manager.set_margin_start(4);
        kill_network_manager.set_margin_end(4);
        kill_network_manager.set_margin_top(4);
        kill_network_manager.set_margin_bottom(4);

        let process_frame = Frame::new(Some("Process"));
        process_frame.set_child(Some(&kill_network_manager));

        let interface_page = page();
        interface_page.append(&mac_frame);
        interface_page.append(&process_frame);

        // --- Scan tab ------------------------------------------------------
        let hop_interval = SpinButton::with_range(50.0, 2000.0, 10.0);
        let refresh_rate = SpinButton::with_range(50.0, 5000.0, 50.0);

        let scan_page = page();
        scan_page.append(&labelled_row(
            "Channel hopping",
            "Hop interval (ms)",
            &hop_interval,
        ));
        scan_page.append(&labelled_row(
            "Interface",
            "Refresh interval (ms)",
            &refresh_rate,
        ));

        // --- Cracking tab --------------------------------------------------
        let wordlist_entry = Entry::builder()
            .placeholder_text("ex: /usr/share/wordlists/rockyou.txt")
            .hexpand(true)
            .editable(true)
            .build();
        let wordlist_but = Button::from_icon_name("edit-find-symbolic");

        let wordlist_box = Box::new(Orientation::Horizontal, 4);
        wordlist_box.set_margin_start(4);
        wordlist_box.set_margin_end(4);
        wordlist_box.set_margin_top(4);
        wordlist_box.set_margin_bottom(4);
        wordlist_box.append(&wordlist_entry);
        wordlist_box.append(&wordlist_but);

        let wordlist_frame = Frame::new(Some("Default wordlist"));
        wordlist_frame.set_child(Some(&wordlist_box));

        let save_dir_entry = Entry::builder()
            .placeholder_text("ex: /home/user/captures")
            .hexpand(true)
            .editable(true)
            .build();
        let save_dir_but = Button::from_icon_name("folder-open-symbolic");

        let save_dir_box = Box::new(Orientation::Horizontal, 4);
        save_dir_box.set_margin_start(4);
        save_dir_box.set_margin_end(4);
        save_dir_box.set_margin_top(4);
        save_dir_box.set_margin_bottom(4);
        save_dir_box.append(&save_dir_entry);
        save_dir_box.append(&save_dir_but);

        let save_dir_frame = Frame::new(Some("Default save directory"));
        save_dir_frame.set_child(Some(&save_dir_box));

        let cracking_page = page();
        cracking_page.append(&wordlist_frame);
        cracking_page.append(&save_dir_frame);

        // --- Display tab ---------------------------------------------------
        let theme = DropDown::from_strings(&THEME_LABELS);
        let theme_frame = labelled_row("Appearance", "Theme", &theme);

        let capture_notifications = CheckButton::with_label("Notify on handshake / PMKID capture");
        capture_notifications.set_active(true);
        capture_notifications.set_margin_start(4);
        capture_notifications.set_margin_end(4);
        capture_notifications.set_margin_top(4);
        capture_notifications.set_margin_bottom(4);

        let notif_frame = Frame::new(Some("Notifications"));
        notif_frame.set_child(Some(&capture_notifications));

        let display_hidden_ap = CheckButton::with_label("Display hidden APs");
        display_hidden_ap.set_active(true);

        let columns_box = Box::new(Orientation::Vertical, 4);
        columns_box.set_margin_start(4);
        columns_box.set_margin_end(4);
        columns_box.set_margin_top(4);
        columns_box.set_margin_bottom(4);
        columns_box.append(&display_hidden_ap);

        let columns: Vec<CheckButton> = COLUMN_LABELS
            .iter()
            .map(|label| {
                let check = CheckButton::with_label(&format!("Show {label} column"));
                check.set_active(true);
                columns_box.append(&check);
                check
            })
            .collect();

        let columns_frame = Frame::new(Some("Access point list"));
        columns_frame.set_child(Some(&columns_box));

        let display_page = page();
        display_page.append(&theme_frame);
        display_page.append(&notif_frame);
        display_page.append(&columns_frame);

        // --- Notebook + footer --------------------------------------------
        let notebook = Notebook::new();
        notebook.set_vexpand(true);
        notebook.append_page(
            &scrollable(&interface_page),
            Some(&Label::new(Some("Interface"))),
        );
        notebook.append_page(&scrollable(&scan_page), Some(&Label::new(Some("Scan"))));
        notebook.append_page(
            &scrollable(&cracking_page),
            Some(&Label::new(Some("Cracking"))),
        );
        notebook.append_page(
            &scrollable(&display_page),
            Some(&Label::new(Some("Display"))),
        );

        let reset_but = Button::with_label("Reset to defaults");
        reset_but.set_halign(Align::Start);
        reset_but.set_hexpand(true);

        let save_but = Button::with_label("Save");
        save_but.set_halign(Align::End);

        let footer = Box::new(Orientation::Horizontal, 10);
        footer.set_margin_start(10);
        footer.set_margin_end(10);
        footer.set_margin_bottom(10);
        footer.append(&reset_but);
        footer.append(&save_but);

        let root = Box::new(Orientation::Vertical, 10);
        root.append(&notebook);
        root.append(&footer);

        window.set_child(Some(&root));

        Self {
            window,
            random_mac,
            default_mac,
            specific_mac,
            mac_entry,
            kill_network_manager,
            hop_interval,
            refresh_rate,
            wordlist_entry,
            wordlist_but,
            save_dir_entry,
            save_dir_but,
            theme,
            capture_notifications,
            display_hidden_ap,
            columns,
            reset_but,
            save_but,
        }
    }

    /// Populate every widget from the given settings. Used both to show the current
    /// settings and to implement "Reset to defaults".
    pub fn apply(&self, settings: &crate::types::Settings) {
        self.mac_entry.set_text("");
        match settings.mac_address.as_str() {
            "random" => self.random_mac.set_active(true),
            "default" => self.default_mac.set_active(true),
            mac => {
                self.specific_mac.set_active(true);
                self.mac_entry.set_text(mac);
            }
        };

        self.kill_network_manager
            .set_active(settings.kill_network_manager);

        self.hop_interval.set_value(settings.hop_interval as f64);
        self.refresh_rate.set_value(settings.refresh_rate as f64);

        self.wordlist_entry.set_text(&settings.wordlist_path);
        self.save_dir_entry.set_text(&settings.save_path);

        self.theme.set_selected(theme_index(settings.theme));
        self.capture_notifications
            .set_active(settings.capture_notifications);
        self.display_hidden_ap
            .set_active(settings.display_hidden_ap);
        self.set_column_checks(&settings.columns);
    }

    /// The theme currently selected in the drop-down.
    pub fn selected_theme(&self) -> Theme {
        match self.theme.selected() {
            1 => Theme::Light,
            2 => Theme::Dark,
            _ => Theme::System,
        }
    }

    /// Set the column check buttons from a [`ColumnVisibility`].
    fn set_column_checks(&self, cols: &ColumnVisibility) {
        let values = [
            cols.bssid,
            cols.band,
            cols.channel,
            cols.power,
            cols.encryption,
            cols.clients,
            cols.first_time_seen,
            cols.last_time_seen,
            cols.handshake,
            cols.pmkid,
        ];
        for (check, value) in self.columns.iter().zip(values) {
            check.set_active(value);
        }
    }

    /// Read the column check buttons into a [`ColumnVisibility`].
    pub fn column_visibility(&self) -> ColumnVisibility {
        let active = |i: usize| {
            self.columns
                .get(i)
                .map(CheckButton::is_active)
                .unwrap_or(true)
        };
        ColumnVisibility {
            bssid: active(0),
            band: active(1),
            channel: active(2),
            power: active(3),
            encryption: active(4),
            clients: active(5),
            first_time_seen: active(6),
            last_time_seen: active(7),
            handshake: active(8),
            pmkid: active(9),
        }
    }

    pub fn show(&self) {
        self.apply(&backend::get_settings());
        self.save_but.set_sensitive(true);
        self.window.show();
    }
}

/// Index of a theme in the drop-down.
fn theme_index(theme: Theme) -> u32 {
    match theme {
        Theme::System => 0,
        Theme::Light => 1,
        Theme::Dark => 2,
    }
}
