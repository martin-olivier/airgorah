use gtk4::prelude::*;
use gtk4::*;
use crate::types::*;

pub struct CaptureGui {
    pub window: Window,
    pub path_entry: Entry,
    pub path_but: Button,
    pub passive_but: CheckButton,
    pub deauth_but: CheckButton,
    pub capture_but: Button,
    pub spinner: Spinner,
}

impl CaptureGui {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = Window::builder()
                .title("Capture Handshake")
                .hide_on_close(true)
                .default_width(400)
                .default_height(140)
                .resizable(false)
                .transient_for(parent)
                .modal(true)
                .build();

        let passive_but = CheckButton::builder()
            .label("Passive mode")
            .tooltip_text("Wait for a client to connect to the access point")
            .build();

        let deauth_but = CheckButton::builder()
            .label("Deauth mode")
            .tooltip_text("Force clients to disconnect and reconnect to capture the handshake")
            .build();

        passive_but.set_active(true);
        deauth_but.set_group(Some(&passive_but));

        passive_but.set_margin_start(15);
        passive_but.set_margin_top(15);

        deauth_but.set_margin_start(15);
        deauth_but.set_margin_bottom(15);

        let but_box = Box::new(Orientation::Vertical, 10);
        but_box.append(&passive_but);
        but_box.append(&deauth_but);

        let frame = Frame::new(None);
        frame.set_child(Some(&but_box));

        let path_entry = Entry::builder()
                .placeholder_text("ex: /root/handshake.cap")
                .hexpand(true)
                .editable(false)
                .build();

        let path_but = Button::from_icon_name("edit-find-symbolic");

        let path_frame = Frame::new(Some("Save to"));

        let path_box = Box::new(Orientation::Horizontal, 4);
        path_box.set_margin_start(4);
        path_box.set_margin_end(4);
        path_box.set_margin_bottom(4);
        path_box.append(&path_entry);
        path_box.append(&path_but);

        path_frame.set_child(Some(&path_box));

        let capture_but = Button::with_label("Start Capture");
        capture_but.set_sensitive(false);

        let spinner = Spinner::new();
        spinner.set_spinning(true);
        spinner.set_sensitive(false);
        spinner.hide();

        let main_box = Box::new(Orientation::Vertical, 10);
        main_box.append(&frame);
        main_box.append(&path_frame);
        main_box.append(&capture_but);
        main_box.append(&spinner);

        main_box.set_margin_bottom(10);
        main_box.set_margin_end(10);
        main_box.set_margin_start(10);
        main_box.set_margin_top(10);

        window.set_child(Some(&main_box));
        
        Self {
            window,
            path_but,
            passive_but,
            deauth_but,
            path_entry,
            capture_but,
            spinner,
        }
    }

    pub fn show(&self, ap: AP) {
        self.window.set_title(Some(&format!("Capture Handshake on \"{}\"", ap.essid)));
        self.path_entry.set_text("");
        self.passive_but.set_sensitive(true);
        self.passive_but.set_active(true);
        self.deauth_but.set_sensitive(true);
        self.deauth_but.set_active(false);
        self.capture_but.set_sensitive(false);
        self.capture_but.set_label("Start Capture");
        self.spinner.hide();
        self.spinner.stop();
        self.path_but.set_sensitive(true);
        self.window.show();
    }
}
