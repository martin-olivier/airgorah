use gtk4::{prelude::*, Adjustment};
use gtk4::{Button, Window};
use std::rc::Rc;

pub fn scan_ui() {
    let win = Window::builder()
        .title("Scan")
        .default_width(200)
        .default_height(170)
        .resizable(false)
        .modal(true)
        .build();

    let hbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

    let ghz_2_4 = gtk4::CheckButton::builder()
        .active(true)
        .label("2.4 GHZ")
        .build();
    let ghz_5 = gtk4::CheckButton::builder()
        .active(false)
        .label("5 GHZ")
        .build();

    ghz_2_4.set_margin_start(6);

    let but_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    but_box.append(&ghz_2_4);
    but_box.append(&ghz_5);

    let band_frame = gtk4::Frame::new(Some("Band"));
    band_frame.set_child(Some(&but_box));

    //

    let channel_filter_entry = Rc::new(
        gtk4::Entry::builder()
            .placeholder_text("ex: 6")
            .sensitive(false)
            .build(),
    );
    let channel_filter_but = Rc::new(gtk4::CheckButton::builder().active(false).build());
    channel_filter_but.set_margin_start(10);

    let channel_filter_entry_other = channel_filter_entry.clone();
    channel_filter_but.connect_toggled(move |this| {
        match this.is_active() {
            true => channel_filter_entry_other.set_sensitive(true),
            false => channel_filter_entry_other.set_sensitive(false),
        };
    });

    //

    let bssid_filter_entry = Rc::new(
        gtk4::Entry::builder()
            .placeholder_text("ex: 10:20:30:40:50")
            .sensitive(false)
            .build(),
    );
    let bssid_filter_but = Rc::new(gtk4::CheckButton::builder().active(false).build());
    bssid_filter_but.set_margin_start(10);

    let bssid_filter_entry_other = bssid_filter_entry.clone();
    bssid_filter_but.connect_toggled(move |this| {
        match this.is_active() {
            true => bssid_filter_entry_other.set_sensitive(true),
            false => bssid_filter_entry_other.set_sensitive(false),
        };
    });

    //

    let channel_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    channel_box.append(channel_filter_but.as_ref());
    channel_box.append(channel_filter_entry.as_ref());

    let bssid_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    bssid_box.append(bssid_filter_but.as_ref());
    bssid_box.append(bssid_filter_entry.as_ref());

    let channel_frame = gtk4::Frame::new(Some("Channel filter"));
    channel_frame.set_child(Some(&channel_box));

    let bssid_frame = gtk4::Frame::new(Some("BSSID filter"));
    bssid_frame.set_child(Some(&bssid_box));

    //

    let spin = gtk4::SpinButton::builder()
        .adjustment(&Adjustment::new(10.0, 5.0, 999.0, 1.0, 10.0, 10.0))
        .build();

    let duration_frame = gtk4::Frame::new(Some("Scan duration (in seconds)"));
    duration_frame.set_child(Some(&spin));

    //

    let keep_old_but = gtk4::CheckButton::with_label("Keep old scan's data");

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

    win.set_child(Some(&hbox));
    win.show();
}
