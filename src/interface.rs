use crate::*;

pub struct Interface {
    pub handle: Window,
    model: gtk4::ListStore,
}

impl Interface {
    pub fn new() -> Self {
        let handle = gtk4::Window::builder()
            .title("Select wireless interface")
            .build();

        let model = gtk4::ListStore::new(&[glib::Type::STRING]);

        model.insert_with_values(Some(0), &[(0, &"wlp3s0")]);
        let combo = gtk4::ComboBox::with_model(&model);

        let cell = gtk4::CellRendererText::new();
        combo.pack_start(&cell, false);
        combo.add_attribute(&cell, "text", 0);
        combo.set_active(Some(0));

        let refresh_but = Button::with_label("Refresh");
            refresh_but.connect_clicked(|but| {
        });

        let select_but = Button::with_label("Select");
        select_but.connect_clicked(|_| {
            eprintln!("selected");
        });

        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 6);

        hbox.append(&combo);
        hbox.append(&refresh_but);

        vbox.append(&hbox);
        vbox.append(&select_but);

        handle.set_child(Some(&vbox));
        handle.show();

        Self { handle, model }
    }

    fn update_interface_list(&self) {
        // TODO
    }
}
