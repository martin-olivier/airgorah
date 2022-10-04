use gtk4::prelude::*;
use gtk4::{Adjustment, Box, Button, CheckButton, Entry, Frame, Orientation, SpinButton, Window};
use std::rc::Rc;

pub struct ScanWindow {
    pub window: Window,
    pub ghz_2_4_but: CheckButton,
    pub ghz_5_but: CheckButton,
    pub channel_filter_but: CheckButton,
    pub channel_filter_entry: Rc<Entry>,
    pub bssid_filter_but: CheckButton,
    pub bssid_filter_entry: Rc<Entry>,
    pub spin: SpinButton,
    pub keep_old_but: CheckButton,
    pub scan_but: Button,
}

impl ScanWindow {
    pub fn new() -> Self {
        let window = Window::builder()
            .title("Scan")
            .default_width(200)
            .default_height(170)
            .resizable(false)
            .modal(true)
            .hide_on_close(true)
            .build();

        let hbox = Box::new(Orientation::Vertical, 10);

        let ghz_2_4_but = CheckButton::builder().active(true).label("2.4 GHZ").build();
        let ghz_5_but = CheckButton::builder().active(false).label("5 GHZ").build();

        ghz_2_4_but.set_margin_start(6);

        let but_box = Box::new(Orientation::Horizontal, 10);
        but_box.append(&ghz_2_4_but);
        but_box.append(&ghz_5_but);

        let band_frame = Frame::new(Some("Band"));
        band_frame.set_child(Some(&but_box));

        //

        let channel_filter_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: 6")
                .sensitive(false)
                .build(),
        );
        let channel_filter_but = CheckButton::builder().active(false).build();
        channel_filter_but.set_margin_start(10);

        let channel_filter_entry_ref = channel_filter_entry.clone();
        channel_filter_but.connect_toggled(move |this| {
            match this.is_active() {
                true => channel_filter_entry_ref.set_sensitive(true),
                false => channel_filter_entry_ref.set_sensitive(false),
            };
        });

        //

        let bssid_filter_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: 10:20:30:40:50")
                .sensitive(false)
                .build(),
        );
        let bssid_filter_but = CheckButton::builder().active(false).build();
        bssid_filter_but.set_margin_start(10);

        let bssid_filter_entry_other = bssid_filter_entry.clone();
        bssid_filter_but.connect_toggled(move |this| {
            match this.is_active() {
                true => bssid_filter_entry_other.set_sensitive(true),
                false => bssid_filter_entry_other.set_sensitive(false),
            };
        });

        //

        let channel_box = Box::new(Orientation::Horizontal, 10);
        channel_box.append(&channel_filter_but);
        channel_box.append(channel_filter_entry.as_ref());

        let bssid_box = Box::new(Orientation::Horizontal, 10);
        bssid_box.append(&bssid_filter_but);
        bssid_box.append(bssid_filter_entry.as_ref());

        let channel_frame = Frame::new(Some("Channel filter"));
        channel_frame.set_child(Some(&channel_box));

        let bssid_frame = Frame::new(Some("BSSID filter"));
        bssid_frame.set_child(Some(&bssid_box));

        //

        let spin = SpinButton::builder()
            .adjustment(&Adjustment::new(10.0, 5.0, 999.0, 1.0, 10.0, 10.0))
            .build();

        let duration_frame = Frame::new(Some("Scan duration (in seconds)"));
        duration_frame.set_child(Some(&spin));

        //

        let keep_old_but = CheckButton::with_label("Keep old scan's data");

        //

        let scan_but = Button::with_label("Scan");

        hbox.append(&band_frame);
        hbox.append(&channel_frame);
        hbox.append(&bssid_frame);
        hbox.append(&duration_frame);
        hbox.append(&keep_old_but);
        hbox.append(&scan_but);

        hbox.set_margin_top(10);
        hbox.set_margin_end(10);
        hbox.set_margin_start(10);
        hbox.set_margin_bottom(10);

        window.set_child(Some(&hbox));

        Self {
            window,
            ghz_2_4_but,
            ghz_5_but,
            channel_filter_but,
            channel_filter_entry,
            bssid_filter_but,
            bssid_filter_entry,
            spin,
            keep_old_but,
            scan_but,
        }
    }
}
