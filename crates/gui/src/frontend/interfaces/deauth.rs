use crate::types::*;

use gtk4::prelude::*;
use gtk4::*;

pub struct DeauthGui {
    pub window: Window,
    pub rate_but: SpinButton,
    pub disassoc_but: CheckButton,
    pub store: ListStore,
    pub view: TreeView,
    pub toggle: CellRendererToggle,
    pub all_cli_but: CheckButton,
    pub sel_cli_but: CheckButton,
    pub attack_but: Button,
}

impl DeauthGui {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = Window::builder()
            .title("Deauth")
            .hide_on_close(true)
            .default_width(300)
            .default_height(400)
            .resizable(false)
            .transient_for(parent)
            .modal(true)
            .build();

        // Injection rate: send rounds per second.
        let rate_label = Label::new(Some("Rate (pkt/s)"));
        rate_label.set_halign(Align::Start);
        rate_label.set_hexpand(true);

        let rate_but = SpinButton::with_range(1.0, 1000.0, 1.0);
        rate_but.set_value(10.0);

        let rate_box = Box::new(Orientation::Horizontal, 10);
        rate_box.append(&rate_label);
        rate_box.append(&rate_but);

        // Optionally send a disassociation frame alongside each deauth.
        let disassoc_but = CheckButton::with_label("Send disassociation frames");

        let settings_box = Box::new(Orientation::Vertical, 10);
        settings_box.append(&rate_box);
        settings_box.append(&disassoc_but);

        settings_box.set_margin_start(10);
        settings_box.set_margin_end(10);
        settings_box.set_margin_top(10);
        settings_box.set_margin_bottom(10);

        let settings_frame = Frame::new(None);
        settings_frame.set_child(Some(&settings_box));

        let all_cli_but = CheckButton::with_label("Deauth all clients");
        let sel_cli_but = CheckButton::with_label("Deauth selected clients");

        all_cli_but.set_active(true);
        sel_cli_but.set_group(Some(&all_cli_but));

        all_cli_but.set_margin_start(15);
        all_cli_but.set_margin_top(15);

        sel_cli_but.set_margin_start(15);
        sel_cli_but.set_margin_bottom(15);

        let store = ListStore::new(&[glib::Type::BOOL, glib::Type::STRING]);

        let column = TreeViewColumn::new();
        column.set_title("Clients");

        let view = TreeView::new();
        view.set_sensitive(false);
        view.set_vexpand(true);
        view.set_model(Some(&store));
        view.append_column(&column);

        let toggle = CellRendererToggle::new();
        column.pack_start(&toggle, false);
        column.add_attribute(&toggle, "active", 0);

        let text_ren = CellRendererText::new();
        column.pack_start(&text_ren, true);
        column.add_attribute(&text_ren, "text", 1);

        let scroll = ScrolledWindow::new();
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_child(Some(&view));

        let deauth_box = Box::new(Orientation::Vertical, 2);
        deauth_box.append(&all_cli_but);
        deauth_box.append(&sel_cli_but);
        deauth_box.append(&scroll);

        let deauth_frame = Frame::new(None);
        deauth_frame.set_child(Some(&deauth_box));

        let attack_but = Button::with_label("Deauth");

        let main_box = Box::new(Orientation::Vertical, 10);
        main_box.append(&settings_frame);
        main_box.append(&deauth_frame);
        main_box.append(&attack_but);

        main_box.set_margin_bottom(10);
        main_box.set_margin_end(10);
        main_box.set_margin_start(10);
        main_box.set_margin_top(10);

        window.set_child(Some(&main_box));

        Self {
            window,
            rate_but,
            disassoc_but,
            store,
            view,
            toggle,
            all_cli_but,
            sel_cli_but,
            attack_but,
        }
    }

    pub fn show(&self, ap: AP) {
        self.window
            .set_title(Some(&format!("Deauth \"{}\"", ap.essid)));

        self.sel_cli_but.set_active(false);
        self.all_cli_but.set_active(true);
        self.view.set_sensitive(false);
        self.attack_but.set_sensitive(true);

        self.store.clear();
        for cli in ap.clients.values() {
            self.store
                .set(&self.store.append(), &[(0, &false), (1, &cli.mac)]);
        }

        self.window.show();
    }
}
