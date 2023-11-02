use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::widgets::*;
use crate::list_store_get;
use crate::types::AttackSoftware;

use glib::clone;
use glib::Value;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;

fn get_selected_clients(storage: &ListStore) -> Vec<String> {
    let mut iter = storage.iter_first();
    let mut selected_clients = vec![];

    while let Some(it) = iter {
        let check_val = list_store_get!(storage, &it, 0, bool);
        let mac_val = list_store_get!(storage, &it, 1, String);

        if check_val {
            selected_clients.push(mac_val);
        }

        iter = match storage.iter_next(&it) {
            true => Some(it),
            false => None,
        }
    }

    selected_clients
}

fn connect_controller(app_data: Rc<AppData>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(clone!(@strong app_data => move |_, key, _, _| {
        if key == gdk::Key::Escape {
            app_data.deauth_gui.window.hide();
        }

        glib::Propagation::Proceed
    }));

    app_data.deauth_gui.window.add_controller(controller);
}

fn connect_all_cli_button(app_data: Rc<AppData>) {
    app_data
        .deauth_gui
        .all_cli_but
        .connect_toggled(clone!(@strong app_data => move |_| {
            app_data.deauth_gui.view.set_sensitive(false);
            app_data.deauth_gui.view.selection().unselect_all();
            app_data.deauth_gui.attack_but.set_sensitive(true);
        }));
}

fn connect_sel_cli_button(app_data: Rc<AppData>) {
    app_data
        .deauth_gui
        .sel_cli_but
        .connect_toggled(clone!(@strong app_data => move |_| {
            app_data.deauth_gui.view.set_sensitive(true);

            if get_selected_clients(&app_data.deauth_gui.store).is_empty() {
                app_data.deauth_gui.attack_but.set_sensitive(false);
            }
        }));
}

fn connect_toggle(app_data: Rc<AppData>) {
    app_data
        .deauth_gui
        .toggle
        .connect_toggled(clone!(@strong app_data => move |_, path| {
            let iter = app_data.deauth_gui.store.iter(&path).unwrap();
            let old_val = list_store_get!(app_data.deauth_gui.store, &iter, 0, bool);

            app_data.deauth_gui.store.set_value(&iter, 0, &Value::from(&(!old_val)));

            match get_selected_clients(&app_data.deauth_gui.store).is_empty() {
                true => app_data.deauth_gui.attack_but.set_sensitive(false),
                false => app_data.deauth_gui.attack_but.set_sensitive(true),
            };
        }));
}

fn connect_attack_but(app_data: Rc<AppData>) {
    app_data.deauth_gui.attack_but.connect_clicked(clone!(@strong app_data => move |_| {
        let params = match app_data.deauth_gui.sel_cli_but.is_active() {
            true => Some(get_selected_clients(&app_data.deauth_gui.store)),
            false => None,
        };

        let iface = match backend::get_iface() {
            Some(iface) => iface,
            None => return,
        };

        let iter = match app_data.app_gui.aps_view.selection().selected() {
            Some((_, iter)) => iter,
            None => return,
        };

        let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);

        let attack_software = match app_data.deauth_gui.aireplay_but.is_active() {
            true => AttackSoftware::Aireplay,
            false => AttackSoftware::Mdk4,
        };

        if attack_software == AttackSoftware::Mdk4 && !backend::has_dependency("mdk4") {
            return ErrorDialog::spawn(
                &app_data.deauth_gui.window,
                "Missing dependency",
                "\"mdk4\" is not installed on your system",
            );
        }

        if let Err(e) = backend::launch_deauth_attack(&iface, backend::get_aps()[&bssid].clone(), params, attack_software) {
            return ErrorDialog::spawn(
                &app_data.deauth_gui.window,
                "Deauth attack failed",
                &format!("Could not start deauth process:\n\n{}", e),
            );
        };

        app_data.deauth_gui.window.hide();
    }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_controller(app_data.clone());

    connect_all_cli_button(app_data.clone());
    connect_sel_cli_button(app_data.clone());
    connect_toggle(app_data.clone());
    connect_attack_but(app_data);
}
