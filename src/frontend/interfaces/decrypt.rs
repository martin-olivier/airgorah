use gtk4::prelude::*;
use gtk4::*;

pub struct DecryptGui {
    pub window: Window,
    pub handshake_but: Button,
    pub handshake_entry: Entry,
    pub wordlist_but: Button,
    pub wordlist_entry: Entry,
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

        let handshake_entry = Entry::builder()
                .placeholder_text("ex: handshake.cap")
                .hexpand(true)
                .editable(false)
                .build();

        let handshake_but = Button::from_icon_name("edit-find-symbolic");

        let handshake_frame = Frame::new(Some("Handshake"));

        let handshake_box = Box::new(Orientation::Horizontal, 4);
        handshake_box.set_margin_start(4);
        handshake_box.set_margin_end(4);
        handshake_box.set_margin_bottom(4);
        handshake_box.append(&handshake_entry);
        handshake_box.append(&handshake_but);

        handshake_frame.set_child(Some(&handshake_box));

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

        let decrypt_but = Button::with_label("Start Decryption");
        decrypt_but.set_sensitive(false);

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.set_margin_top(10);
        vbox.set_margin_end(10);
        vbox.set_margin_start(10);
        vbox.set_margin_bottom(10);

        vbox.append(&handshake_frame);
        vbox.append(&wordlist_frame);
        vbox.append(&decrypt_but);

        window.set_child(Some(&vbox));

        Self {
            window,
            handshake_but,
            handshake_entry,
            wordlist_but,
            wordlist_entry,
            decrypt_but,
        }
    }

    pub fn show(&self, capture_file: Option<String>) {
        self.handshake_entry.set_text("");
        self.wordlist_entry.set_text("");
        self.decrypt_but.set_sensitive(false);

        if let Some(path) = capture_file {
            self.handshake_entry.set_text(&path);
        }

        self.window.show();
    }
}
