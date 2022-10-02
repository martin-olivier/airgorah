use gtk4::prelude::*;
use gtk4::{Application, Button, ListStore, Window};
use std::rc::Rc;

use crate::globals::IFACE;

pub fn interfaces_ui(app: &Application) {
    let handle = Window::builder()
        .title("Select wireless interface")
        .default_width(350)
        .default_height(70)
        .resizable(false)
        .modal(true)
        .build();

    handle.set_transient_for(app.active_window().as_ref());

    let model = Rc::new(ListStore::new(&[glib::Type::STRING]));

    let ifaces = crate::backend::get_interfaces();
    for iface in ifaces.into_iter() {
        model.insert_with_values(None, &[(0, &iface)]);
    }

    let combo = gtk4::ComboBox::with_model(&*model);
    combo.set_width_request(240);

    let cell = gtk4::CellRendererText::new();
    combo.pack_start(&cell, false);
    combo.add_attribute(&cell, "text", 0);
    combo.set_active(Some(0));

    let refresh_but = Button::with_label("Refresh");
    let select_but = Button::with_label("Select");

    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

    hbox.append(&combo);
    hbox.append(&refresh_but);

    hbox.set_margin_top(10);
    hbox.set_margin_start(10);
    hbox.set_margin_end(10);

    vbox.set_margin_top(0);

    vbox.append(&hbox);
    vbox.append(&select_but);

    handle.set_child(Some(&vbox));
    handle.show();

    let modeles = model.clone();

    refresh_but.connect_clicked(move |_| {
        model.clear();

        let ifaces = crate::backend::get_interfaces();
        for iface in ifaces.into_iter() {
            model.insert_with_values(None, &[(0, &iface)]);
        }
    });

    select_but.connect_clicked(move |_| {
        let iter = match combo.active_iter() {
            Some(iter) => iter,
            None => return,
        };
        let val = modeles.get_value(&iter, 0);
        let iface = val.get::<&str>().unwrap();

        match crate::backend::enable_monitor_mode(iface) {
            Ok(str) => {
                IFACE.lock().unwrap().clear();
                IFACE.lock().unwrap().push_str(str.as_str());
                handle.close();
            }
            Err(()) => {
                let dialog = gtk4::MessageDialog::builder()
                    .text(&format!("Monitor mode failed"))
                    .secondary_text(&format!("Could not enable monitor mode on \"{}\"", iface))
                    .decorated(true)
                    .message_type(gtk4::MessageType::Error)
                    .buttons(gtk4::ButtonsType::Close)
                    .modal(true)
                    .transient_for(&handle)
                    .build();
                dialog.show();
                dialog.connect_response(|this, _| {
                    this.close();
                });
            }
        };
    });
}
