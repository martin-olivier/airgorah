use super::dialog::*;
use crate::backend;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;

pub struct InterfaceWindow {
    pub window: Rc<Window>,
    pub select_but: Button,
    pub refresh_but: Button,
    pub combo: Rc<ComboBox>,
    pub model: Rc<ListStore>,
}

impl InterfaceWindow {
    pub fn new(app: &Application) -> Self {
        let window = Rc::new(
            Window::builder()
                .title("Select a wireless interface")
                .hide_on_close(true)
                .default_width(280)
                .default_height(70)
                .resizable(false)
                .modal(true)
                .build(),
        );
        window.set_transient_for(app.active_window().as_ref());

        let model = Rc::new(ListStore::new(&[glib::Type::STRING]));

        let cell = CellRendererText::new();

        let combo = Rc::new(ComboBox::with_model(model.as_ref()));
        combo.set_hexpand(true);
        combo.pack_start(&cell, false);
        combo.add_attribute(&cell, "text", 0);

        let refresh_but = Button::builder().icon_name("object-rotate-right").build();
        let select_but = Button::with_label("Select");

        let hbox = Box::new(Orientation::Horizontal, 10);
        let vbox = Box::new(Orientation::Vertical, 10);

        hbox.append(combo.as_ref());
        hbox.append(&refresh_but);

        hbox.set_margin_top(10);
        hbox.set_margin_start(10);
        hbox.set_margin_end(10);

        vbox.append(&hbox);
        vbox.append(&select_but);

        vbox.set_margin_top(0);

        window.set_child(Some(&vbox));
        window.show();

        let window_ref = window.clone();
        let model_ref = model.clone();
        let combo_ref = combo.clone();

        refresh_but.connect_clicked(move |_| {
            model_ref.clear();

            let ifaces = match backend::get_interfaces() {
                Ok(ifaces) => ifaces,
                Err(e) => {
                    return ErrorDialog::spawn(
                        window_ref.as_ref(),
                        "Failed to get interfaces",
                        &e.to_string(),
                        false,
                    );
                }
            };

            for iface in ifaces.iter() {
                model_ref.insert_with_values(None, &[(0, &iface)]);
            }

            combo_ref.set_active(if !ifaces.is_empty() { Some(0) } else { None });
        });

        refresh_but.emit_clicked();

        Self {
            window,
            select_but,
            refresh_but,
            combo,
            model,
        }
    }
}
