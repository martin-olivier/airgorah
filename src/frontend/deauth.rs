use glib::Value;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;

use super::dialog::*;
use crate::backend;
use crate::list_store_get;
use crate::types::*;

pub struct DeauthWindow;

fn get_selected_clis(storage: &ListStore) -> Vec<String> {
    let mut iter = storage.iter_first();
    let mut selected_clis = vec![];

    while let Some(it) = iter {
        let check_val = list_store_get!(storage, &it, 0, bool);
        let mac_val = list_store_get!(storage, &it, 1, String);

        if check_val {
            selected_clis.push(mac_val);
        }

        iter = match storage.iter_next(&it) {
            true => Some(it),
            false => None,
        }
    }

    selected_clis
}

impl DeauthWindow {
    pub fn spawn(parent: &impl IsA<Window>, ap: AP) {
        let window = Rc::new(
            Window::builder()
                .title(&format!("Deauth \"{}\"", ap.essid))
                .default_width(360)
                .default_height(140)
                .resizable(false)
                .modal(true)
                .build(),
        );

        window.set_transient_for(Some(parent));

        let all_cli_but = CheckButton::with_label("Deauth all clients");
        let sel_cli_but = CheckButton::with_label("Deauth selected clients");

        all_cli_but.set_active(true);
        sel_cli_but.set_group(Some(&all_cli_but));

        all_cli_but.set_margin_start(15);
        all_cli_but.set_margin_top(15);

        sel_cli_but.set_margin_start(15);
        sel_cli_but.set_margin_bottom(15);

        let but_box = Box::new(Orientation::Vertical, 10);
        but_box.append(&all_cli_but);
        but_box.append(&sel_cli_but);

        let store = Rc::new(ListStore::new(&[glib::Type::BOOL, glib::Type::STRING]));

        let frame = Frame::new(None);
        frame.set_child(Some(&but_box));

        let view = Rc::new(TreeView::new());
        view.set_model(Some(store.as_ref()));

        for cli in ap.clients.iter() {
            store.set(&store.append(), &[(0, &false), (1, &cli.mac)]);
        }

        let column = TreeViewColumn::new();
        column.set_title("Clients");
        view.append_column(&column);
        view.set_vexpand(true);

        let toggle = CellRendererToggle::new();
        column.pack_start(&toggle, false);
        column.add_attribute(&toggle, "active", 0);

        let text_ren = CellRendererText::new();
        column.pack_start(&text_ren, true);
        column.add_attribute(&text_ren, "text", 1);

        let scroll = Rc::new(ScrolledWindow::new());
        scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
        scroll.set_child(Some(view.as_ref()));
        scroll.hide();

        let attack_but = Rc::new(Button::with_label("Attack"));

        let main_box = Box::new(Orientation::Vertical, 10);
        main_box.append(&frame);
        main_box.append(&*scroll);
        main_box.append(&*attack_but);

        main_box.set_margin_bottom(10);
        main_box.set_margin_end(10);
        main_box.set_margin_start(10);
        main_box.set_margin_top(10);

        window.set_child(Some(&main_box));
        window.show();

        // Callbacks

        let window_ref = window.clone();
        let scroll_ref = scroll.clone();
        let attack_but_ref = attack_but.clone();

        all_cli_but.connect_toggled(move |_| {
            scroll_ref.hide();
            window_ref.set_height_request(140);
            attack_but_ref.set_sensitive(true);
        });

        let window_ref = window.clone();
        let store_ref = store.clone();
        let attack_but_ref = attack_but.clone();

        sel_cli_but.connect_toggled(move |_| {
            scroll.show();
            window_ref.set_height_request(300);
            if get_selected_clis(&store_ref).is_empty() {
                attack_but_ref.set_sensitive(false);
            }
        });

        let store_ref = store.clone();
        let attack_but_ref = attack_but.clone();

        toggle.connect_toggled(move |_, path| {
            let iter = store_ref.iter(&path).unwrap();
            let old_val = list_store_get!(store_ref, &iter, 0, bool);

            store_ref.set_value(&iter, 0, &Value::from(&(!old_val)));

            match get_selected_clis(&store_ref).is_empty() {
                true => attack_but_ref.set_sensitive(false),
                false => attack_but_ref.set_sensitive(true),
            };
        });

        attack_but.connect_clicked(move |_| {
            let params = match sel_cli_but.is_active() {
                true => Some(get_selected_clis(store.as_ref())),
                false => None,
            };

            backend::launch_deauth_attack(ap.clone(), params).unwrap_or_else(|e| {
                return ErrorDialog::spawn(
                    window.as_ref(),
                    "Error",
                    &format!("Could not start deauth process: {}", e),
                    false,
                );
            });

            window.close();
        });
    }
}
