mod interface;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
use gtk4 as gtk;
use std::rc::Rc;

fn build_aps_view(view: &gtk4::TreeView) {
    let colomn_names = ["ESSID", "BSSID", "Channel", "Speed", "Power", "Encryption"];
    let mut pos = 0;

    for colomn_name in colomn_names {
        let column = gtk4::TreeViewColumn::builder()
            .title(colomn_name)
            .resizable(true)
            .min_width(50)
            .sort_indicator(true)
            .build();
        view.append_column(&column);

        let renderer = gtk4::CellRendererText::new();
        column.pack_start(&renderer, true);
        column.add_attribute(&renderer, "text", pos);
        pos += 1;
    }
    //renderer2.set_background(Some("Orange"));
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Airgorah")
        .default_width(850)
        .default_height(370)
        .build();

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

    let model = Rc::new(gtk4::ListStore::new(&[
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
    ]));

    let it = model.append();
    model.set(
        &it,
        &[
            (0, &"NEUF_A598".to_string()),
            (1, &"10:20:30:40:50".to_string()),
            (2, &"6".to_string()),
            (3, &"55".to_string()),
            (4, &"95".to_string()),
            (5, &"WPA2".to_string()),
        ],
    );

    let view = gtk4::TreeView::new();
    build_aps_view(&view);

    view.set_vexpand(true);
    view.set_model(Some(&*model));

    let button = Button::with_label("About");
    button.connect_clicked(|_| {
        let about = gtk4::AboutDialog::new();
        about.show();
    });

    main_box.append(&view);
    main_box.append(&button);

    window.set_child(Some(&main_box));
    window.show();

    interface::interfaces_ui(&app);
}
