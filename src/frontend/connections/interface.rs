use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::*;
use crate::list_store_get;

use glib::clone;
use gtk4::prelude::*;
use std::rc::Rc;

fn connect_interface_refresh(app_data: Rc<AppData>) {
    app_data
        .interface_gui
        .refresh_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            app_data.interface_gui.interface_model.clear();

            let ifaces = match backend::get_interfaces() {
                Ok(ifaces) => ifaces,
                Err(e) => {
                    return ErrorDialog::spawn(
                        &app_data.interface_gui.window,
                        "Failed to get interfaces",
                        &e.to_string(),
                        false,
                    );
                }
            };

            for iface in ifaces.iter() {
                app_data.interface_gui.interface_model.insert_with_values(None, &[(0, &iface)]);
            }

            app_data.interface_gui.interface_view.set_active(if !ifaces.is_empty() { Some(0) } else { None });
            app_data.interface_gui.select_but.set_sensitive(!ifaces.is_empty());
        }));
    app_data.interface_gui.refresh_but.emit_clicked();
}

fn connect_interface_select(app_data: Rc<AppData>) {
    app_data
        .interface_gui
        .select_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let iter = match app_data.interface_gui.interface_view.active_iter() {
                Some(iter) => iter,
                None => return,
            };
            let iface = list_store_get!(app_data.interface_gui.interface_model, &iter, 0, String);

            match crate::backend::enable_monitor_mode(&iface) {
                Ok(res) => {
                    backend::set_iface(res.clone());

                    app_data.app_gui.iface_label.set_text(&res);
                    app_data.interface_gui.window.hide();
                    app_data.app_gui.scan_but.emit_clicked();
                }
                Err(e) => {
                    ErrorDialog::spawn(
                        &app_data.interface_gui.window,
                        "Monitor mode failed",
                        &format!("Could not enable monitor mode on \"{}\":\n{}", iface, e),
                        false,
                    );
                    app_data.interface_gui.refresh_but.emit_clicked();
                }
            };
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_interface_refresh(app_data.clone());
    connect_interface_select(app_data);
}
