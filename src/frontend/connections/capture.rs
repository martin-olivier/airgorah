use crate::backend;
use crate::frontend::*;
use crate::frontend::interfaces::*;
use crate::list_store_get;
use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use std::time::Duration;

fn connect_path_button(app_data: Rc<AppData>) {
    let iter = match app_data.app_gui.aps_view.selection().selected() {
        Some((_, iter)) => iter,
        None => return,
    };

    let network_name = list_store_get!(app_data.app_gui.aps_model, &iter, 0, String);

    app_data.capture_gui.path_but.connect_clicked(
            clone!(@strong app_data => move |_| {
                let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                    Some("Save Capture"),
                    Some(&app_data.capture_gui.window),
                    FileChooserAction::Save,
                    &[("Save", ResponseType::Accept)],
                ));

                file_chooser_dialog.set_current_name(&(network_name.clone() + ".cap"));
                file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                    if response == ResponseType::Accept {
                        let gio_file = match this.file() {
                            Some(file) => file,
                            None => return,
                        };
                        app_data.capture_gui.path_entry.set_text(gio_file.path().unwrap().to_str().unwrap());
                        app_data.capture_gui.capture_but.set_sensitive(true);
                    }
                    this.close();
                }));
            }),
        );
}

fn connect_capture_button(app_data: Rc<AppData>) {
    let iter = match app_data.app_gui.aps_view.selection().selected() {
        Some((_, iter)) => iter,
        None => return,
    };
    let bssid = list_store_get!(app_data.app_gui.aps_model, &iter, 1, String);
    let ap = backend::get_aps()[&bssid].clone();

    app_data.capture_gui.capture_but.connect_clicked(clone!(@strong app_data => move |this| {
        if backend::is_capture_process() {
            app_data.capture_gui.spinner.hide();
            app_data.capture_gui.spinner.stop();
            app_data.capture_gui.passive_but.set_sensitive(true);
            app_data.capture_gui.deauth_but.set_sensitive(true);
            app_data.capture_gui.path_but.set_sensitive(true);
            this.set_label("Start Capture");

            backend::stop_capture_process();
            if app_data.capture_gui.deauth_but.is_active() {
                backend::stop_deauth_attack(&ap.bssid);
            }
        } else {
            app_data.capture_gui.spinner.show();
            app_data.capture_gui.spinner.start();
            app_data.capture_gui.passive_but.set_sensitive(false);
            app_data.capture_gui.deauth_but.set_sensitive(false);
            app_data.capture_gui.path_but.set_sensitive(false);
            this.set_label("Stop Capture");

            backend::set_capture_process(ap.clone()).unwrap();
            if app_data.capture_gui.deauth_but.is_active() {
                backend::launch_deauth_attack(ap.clone(), None).unwrap();
            }

            glib::timeout_add_local(Duration::from_secs(1), clone!(@strong app_data, @strong ap => move || {
                if !backend::is_capture_process() {
                    return glib::Continue(false);
                }

                if backend::has_handshake().unwrap() {
                    let path = app_data.capture_gui.path_entry.text();
                    backend::save_capture(&path);
                    
                    YesNoDialog::spawn(&app_data.capture_gui.window, "Handshake Captured", "Handshake has been captured!\nWould you like to decrypt the password now?", clone!(@strong app_data => move |this, response| {
                        if response == ResponseType::Yes {
                            app_data.decrypt_gui.show(Some(path.to_string()));
                        }
                        this.close();
                    }));

                    backend::stop_capture_process();
                    if app_data.capture_gui.deauth_but.is_active() {
                        backend::stop_deauth_attack(&ap.bssid);
                    }
                    app_data.capture_gui.window.hide();
                    return glib::Continue(false);
                }
                glib::Continue(true)
            }));
        }
    }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_path_button(app_data.clone());
    connect_capture_button(app_data.clone());
}