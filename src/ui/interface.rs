use gtk4::prelude::*;
use gtk4::{Application, Box, Button, CellRendererText, ComboBox, ListStore, Orientation, Window};
use std::rc::Rc;

pub struct InterfaceWindow {
    pub window: Window,
    pub select_but: Button,
    pub combo: ComboBox,
    pub model: Rc<ListStore>,
}

impl InterfaceWindow {
    pub fn new(app: &Application) -> Self {
        let window = Window::builder()
            .title("Select wireless interface")
            .default_width(350)
            .default_height(70)
            .resizable(false)
            .modal(true)
            .build();

        window.set_transient_for(app.active_window().as_ref());

        let model = Rc::new(ListStore::new(&[glib::Type::STRING]));

        let ifaces = crate::backend::get_interfaces();
        for iface in ifaces.into_iter() {
            model.insert_with_values(None, &[(0, &iface)]);
        }

        let combo = ComboBox::with_model(&*model);
        combo.set_width_request(240);

        let cell = CellRendererText::new();
        combo.pack_start(&cell, false);
        combo.add_attribute(&cell, "text", 0);
        combo.set_active(Some(0));

        let refresh_but = Button::with_label("Refresh");
        let select_but = Button::with_label("Select");

        let hbox = Box::new(Orientation::Horizontal, 10);
        let vbox = Box::new(Orientation::Vertical, 10);

        hbox.append(&combo);
        hbox.append(&refresh_but);

        hbox.set_margin_top(10);
        hbox.set_margin_start(10);
        hbox.set_margin_end(10);

        vbox.set_margin_top(0);

        vbox.append(&hbox);
        vbox.append(&select_but);

        window.set_child(Some(&vbox));
        window.show();

        let model_ref = model.clone();
        refresh_but.connect_clicked(move |_| {
            model_ref.clear();

            let ifaces = crate::backend::get_interfaces();
            for iface in ifaces.into_iter() {
                model_ref.insert_with_values(None, &[(0, &iface)]);
            }
        });
        Self {
            window,
            select_but,
            combo,
            model,
        }
    }
}
