use crate::backend;
use crate::frontend::interfaces::*;
use crate::list_store_get;
use glib::clone;
use gtk4::prelude::*;
use std::rc::Rc;

pub fn connect_interface_refresh(app_data: Rc<AppData>) {
    app_data
        .refresh_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            app_data.interface_model.clear();

            let ifaces = match backend::get_interfaces() {
                Ok(ifaces) => ifaces,
                Err(e) => {
                    return ErrorDialog::spawn(
                        &app_data.interface_window,
                        "Failed to get interfaces",
                        &e.to_string(),
                        false,
                    );
                }
            };

            for iface in ifaces.iter() {
                app_data.interface_model.insert_with_values(None, &[(0, &iface)]);
            }

            app_data.interface_view.set_active(if !ifaces.is_empty() { Some(0) } else { None });
        }));
    app_data.refresh_but.emit_clicked();
}

pub fn connect_interface_select(app_data: Rc<AppData>) {
    app_data
        .select_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.interface_view.active_iter() {
                Some(iter) => iter,
                None => return,
            };
            let iface = list_store_get!(app_data.interface_model, &iter, 0, String);

            match crate::backend::enable_monitor_mode(&iface) {
                Ok(res) => {
                    backend::set_iface(res.clone());

                    app_data.iface_label.set_text(&res);
                    app_data.interface_window.hide();
                    app_data.scan_but.emit_clicked();
                }
                Err(e) => {
                    ErrorDialog::spawn(
                        &app_data.interface_window,
                        "Monitor mode failed",
                        &format!("Could not enable monitor mode on \"{}\":\n{}", iface, e),
                        false,
                    );
                    app_data.refresh_but.emit_clicked();
                }
            };
        }));
}
