use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::*;
use crate::list_store_get;

use glib::clone;
use gtk4::prelude::*;
use std::rc::Rc;

fn connect_controller(app_data: Rc<AppData>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(clone!(@strong app_data => move |_, key, _, _| {
        if key == gdk::Key::Escape {
            app_data.interface_gui.window.hide();
        }

        glib::Propagation::Proceed
    }));

    app_data.interface_gui.window.add_controller(controller);
}

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
                    );
                }
            };

            for iface in ifaces.iter() {
                app_data.interface_gui.interface_model.insert_with_values(None, &[(0, &iface)]);
            }

            app_data.interface_gui.interface_view.set_active(if !ifaces.is_empty() { Some(0) } else { None });
            app_data.interface_gui.select_but.set_sensitive(!ifaces.is_empty());
        }));
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

            match backend::enable_monitor_mode(&iface) {
                Ok(iface) => {
                    if let Err(e) = backend::set_mac_address(&iface) {
                        backend::disable_monitor_mode(&iface).ok();

                        app_data.interface_gui.refresh_but.emit_clicked();

                        return ErrorDialog::spawn(
                            &app_data.interface_gui.window,
                            "Failed to set MAC address",
                            &e.to_string(),
                        );
                    }

                    backend::set_iface(iface.clone());

                    app_data.app_gui.iface_status_bar.push(0, &format!("Interface: {iface}"));

                    app_data.app_gui.restart_but.set_sensitive(true);
                    app_data.app_gui.channel_filter_entry.set_sensitive(true);

                    app_data.app_gui.scan_but.emit_clicked();

                    match backend::is_5ghz_supported(&iface).unwrap_or(false) {
                        true => {
                            app_data.app_gui.ghz_2_4_but.set_sensitive(true);
                            app_data.app_gui.ghz_5_but.set_sensitive(true);
                            app_data.app_gui.ghz_5_but.set_active(true);
                        }
                        false => app_data.app_gui.ghz_5_but.set_tooltip_text(Some(
                            "Your network card doesn't support 5 GHz"
                        ))
                    }

                    app_data.interface_gui.window.hide();
                }
                Err(e) => {
                    app_data.interface_gui.refresh_but.emit_clicked();

                    ErrorDialog::spawn(
                        &app_data.interface_gui.window,
                        "Monitor mode failed",
                        &e.to_string(),
                    );
                }
            };
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_controller(app_data.clone());

    connect_interface_refresh(app_data.clone());
    connect_interface_select(app_data);
}
