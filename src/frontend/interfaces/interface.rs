use gtk4::prelude::*;
use gtk4::*;

fn build_interface_model() -> ListStore {
    ListStore::new(&[glib::Type::STRING])
}

fn build_interface_view(model: &ListStore) -> ComboBox {
    let text_renderer = CellRendererText::new();

    let icon_renderer = CellRendererPixbuf::new();
    icon_renderer.set_property("icon-name", "network-wired");

    let combo = ComboBox::with_model(model);
    combo.set_hexpand(true);
    combo.pack_start(&icon_renderer, false);
    combo.pack_start(&text_renderer, false);
    combo.add_attribute(&text_renderer, "text", 0);

    combo
}

pub struct InterfaceGui {
    pub window: Window,
    pub select_but: Button,
    pub refresh_but: Button,
    pub interface_model: ListStore,
    pub interface_view: ComboBox,
}

impl InterfaceGui {
    pub fn new(parent: &impl IsA<Window>) -> Self {
        let window = Window::builder()
            .title("Select a wireless interface")
            .hide_on_close(true)
            .default_width(280)
            .default_height(70)
            .resizable(false)
            .modal(true)
            .transient_for(parent)
            .build();

        let interface_model = build_interface_model();
        let interface_view = build_interface_view(&interface_model);

        let refresh_but = Button::builder().icon_name("view-refresh-symbolic").build();

        let select_but = Button::with_label("Select");

        let hbox = Box::new(Orientation::Horizontal, 10);
        let vbox = Box::new(Orientation::Vertical, 10);

        hbox.append(&interface_view);
        hbox.append(&refresh_but);

        vbox.set_margin_top(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);
        vbox.set_margin_bottom(10);

        vbox.append(&hbox);
        vbox.append(&select_but);

        window.set_child(Some(&vbox));

        Self {
            window,
            select_but,
            refresh_but,
            interface_model,
            interface_view,
        }
    }
}
