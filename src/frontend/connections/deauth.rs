use crate::backend;
use crate::frontend::interfaces::*;
use crate::list_store_get;
use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;

pub fn connect_deauth_button(app_data: Rc<AppData>) {
    app_data
        .deauth_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.aps_view.selection().selected() {
                Some((_, iter)) => iter,
                None => return,
            };

            let bssid = list_store_get!(app_data.aps_model, &iter, 1, String);
            let under_attack = backend::get_attack_pool().contains_key(&bssid);

            match under_attack {
                true => backend::stop_deauth_attack(&bssid),
                false => {
                    if backend::is_scan_process() {
                        app_data.scan_but.emit_clicked();
                    }
                    DeauthWindow::spawn(
                        &app_data.main_window,
                        backend::get_aps()[&bssid].clone(),
                    );
                }
            }
        }));
}
