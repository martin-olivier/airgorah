use crate::backend;
use crate::frontend::ErrorDialog;

use gtk4::prelude::*;
use gtk4::*;

fn build_ap_model() -> ListStore {
    ListStore::new(&[glib::Type::STRING, glib::Type::STRING])
}

fn build_ap_view(model: &ListStore) -> ComboBox {
    let text_renderer = CellRendererText::new();

    let icon_renderer = CellRendererPixbuf::new();
    icon_renderer.set_property("icon-name", "network-wireless");

    let combo = ComboBox::with_model(model);
    combo.set_hexpand(true);
    combo.pack_start(&icon_renderer, false);
    combo.pack_start(&text_renderer, false);
    combo.add_attribute(&text_renderer, "text", 1);

    combo
}

pub struct DecryptGui {
    pub window: Window,
    pub handshake_but: Button,
    pub handshake_entry: Entry,
    pub target_model: ListStore,
    pub target_view: ComboBox,
    pub stack: Stack,
    pub wordlist_but: Button,
    pub wordlist_entry: Entry,
    pub lowercase_but: CheckButton,
    pub uppercase_but: CheckButton,
    pub numbers_but: CheckButton,
    pub symbols_but: CheckButton,
    pub decrypt_but: Button,
}

impl DecryptGui {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = Window::builder()
            .title("Decrypt Handshake")
            .hide_on_close(true)
            .default_width(500)
            .default_height(200)
            .resizable(false)
            .transient_for(parent)
            .modal(true)
            .build();

        //

        let wordlist_entry = Entry::builder()
            .placeholder_text("ex: rockyou.txt")
            .hexpand(true)
            .editable(false)
            .build();

        let wordlist_but = Button::from_icon_name("edit-find-symbolic");

        let wordlist_frame = Frame::new(Some("Wordlist"));

        let wordlist_box = Box::new(Orientation::Horizontal, 4);
        wordlist_box.set_margin_start(4);
        wordlist_box.set_margin_end(4);
        wordlist_box.set_margin_bottom(4);
        wordlist_box.append(&wordlist_entry);
        wordlist_box.append(&wordlist_but);

        wordlist_frame.set_child(Some(&wordlist_box));

        //

        let lowercase_but = CheckButton::with_label("Lowercase [a-z]");
        let uppercase_but = CheckButton::with_label("Uppercase [A-Z]");
        let numbers_but = CheckButton::with_label("Numbers [0-9]");
        let symbols_but = CheckButton::with_label("Symbols [!-#]");

        //

        let bruteforce_frame = Frame::new(Some("Charset"));

        let bruteforce_box = Box::new(Orientation::Horizontal, 4);
        bruteforce_box.set_margin_start(4);
        bruteforce_box.set_margin_end(4);
        bruteforce_box.set_margin_bottom(4);
        bruteforce_box.append(&lowercase_but);
        bruteforce_box.append(&uppercase_but);
        bruteforce_box.append(&numbers_but);
        bruteforce_box.append(&symbols_but);

        bruteforce_frame.set_child(Some(&bruteforce_box));

        //

        let stack = Stack::new();

        stack.add_titled(&wordlist_frame, Some("dictionary"), "Dictionary");
        stack.add_titled(&bruteforce_frame, Some("bruteforce"), "Bruteforce");

        let stack_switcher = StackSwitcher::new();
        stack_switcher.set_stack(Some(&stack));

        //

        let handshake_entry = Entry::builder()
            .placeholder_text("ex: handshake.cap")
            .hexpand(true)
            .editable(false)
            .build();

        let handshake_but = Button::from_icon_name("edit-find-symbolic");

        let handshake_frame = Frame::new(Some("Capture"));

        let handshake_box = Box::new(Orientation::Horizontal, 4);
        handshake_box.set_margin_start(4);
        handshake_box.set_margin_end(4);
        handshake_box.set_margin_bottom(4);
        handshake_box.append(&handshake_entry);
        handshake_box.append(&handshake_but);

        handshake_frame.set_child(Some(&handshake_box));

        //

        let target_model = build_ap_model();
        let target_view = build_ap_view(&target_model);

        let target_frame = Frame::new(Some("Target"));

        let target_box = Box::new(Orientation::Horizontal, 4);
        target_box.set_margin_start(4);
        target_box.set_margin_end(4);
        target_box.set_margin_bottom(4);
        target_box.append(&target_view);

        target_frame.set_child(Some(&target_box));

        //

        let decrypt_but = Button::with_label("Start Decryption");
        decrypt_but.set_sensitive(false);

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.set_margin_top(10);
        vbox.set_margin_end(10);
        vbox.set_margin_start(10);
        vbox.set_margin_bottom(10);

        vbox.append(&stack_switcher);
        vbox.append(&stack);
        vbox.append(&handshake_frame);
        vbox.append(&target_frame);
        vbox.append(&decrypt_but);

        window.set_child(Some(&vbox));

        Self {
            window,
            handshake_but,
            handshake_entry,
            target_model,
            target_view,
            stack,
            wordlist_but,
            wordlist_entry,
            lowercase_but,
            uppercase_but,
            numbers_but,
            symbols_but,
            decrypt_but,
        }
    }

    pub fn show(&self, capture_and_bssid: Option<(String, String)>) {
        self.handshake_entry.set_text("");
        self.handshake_entry.set_sensitive(true);
        self.handshake_but.set_sensitive(true);
        self.target_view.set_active(None);
        self.target_model.clear();
        self.target_view.set_sensitive(true);
        self.wordlist_entry.set_text("");
        self.lowercase_but.set_active(false);
        self.uppercase_but.set_active(false);
        self.numbers_but.set_active(false);
        self.symbols_but.set_active(false);
        self.decrypt_but.set_sensitive(false);

        if let Some((path, bssid)) = capture_and_bssid {
            self.handshake_entry.set_text(&path);

            let handshakes = backend::get_handshakes([&path]).unwrap_or_default();

            if handshakes.is_empty() {
                return ErrorDialog::spawn(
                    &self.window,
                    "Invalid capture",
                    &format!("\"{}\" doesn't contain any valid handshake", path),
                );
            }

            for (hs_bssid, hs_essid) in handshakes.iter() {
                if hs_bssid == &bssid {
                    self.target_model
                        .insert_with_values(None, &[(0, &hs_bssid), (1, &hs_essid)]);
                }
            }

            self.handshake_entry.set_sensitive(false);
            self.handshake_but.set_sensitive(false);
            self.target_view.set_active(Some(0));
            self.target_view.set_sensitive(false);
        }

        self.window.show();
    }
}
