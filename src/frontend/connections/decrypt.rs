use crate::backend;
use crate::frontend::interfaces::*;
use crate::frontend::widgets::ErrorDialog;
use crate::*;

use glib::clone;
use gtk4::*;
use std::rc::Rc;

fn connect_controller(app_data: Rc<AppData>) {
    let controller = gtk4::EventControllerKey::new();

    controller.connect_key_pressed(clone!(@strong app_data => move |_, key, _, _| {
        if key == gdk::Key::Escape {
            app_data.decrypt_gui.window.hide();
        }

        glib::Propagation::Proceed
    }));

    app_data.decrypt_gui.window.add_controller(controller);
}

fn update_decrypt_button_status(app_data: Rc<AppData>) {
    if app_data.decrypt_gui.handshake_entry.text_length() == 0 {
        return app_data.decrypt_gui.decrypt_but.set_sensitive(false);
    }

    let stack = app_data.decrypt_gui.stack.visible_child_name().unwrap();

    if stack == "dictionary" {
        if app_data.decrypt_gui.wordlist_entry.text_length() == 0 {
            return app_data.decrypt_gui.decrypt_but.set_sensitive(false);
        }
    } else {
        let low = app_data.decrypt_gui.lowercase_but.is_active();
        let up = app_data.decrypt_gui.uppercase_but.is_active();
        let num = app_data.decrypt_gui.numbers_but.is_active();
        let sym = app_data.decrypt_gui.symbols_but.is_active();

        if !low && !up && !num && !sym {
            return app_data.decrypt_gui.decrypt_but.set_sensitive(false);
        }
    }

    app_data.decrypt_gui.decrypt_but.set_sensitive(true);
}

fn connect_handshake_button(app_data: Rc<AppData>) {
    app_data.decrypt_gui.handshake_but.connect_clicked(
        clone!(@strong app_data => move |_| {

            let file_chooser_dialog = FileChooserDialog::new(
                Some("Select Capture"),
                Some(&app_data.decrypt_gui.window),
                FileChooserAction::Open,
                &[("Open", ResponseType::Accept)],
            );

            file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                this.close();

                if response == ResponseType::Accept {
                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };

                    let gio_path = gio_file.path().unwrap();
                    let file_path = gio_path.to_str().unwrap();

                    let handshakes = backend::get_handshakes([file_path]).unwrap_or_default();

                    if handshakes.is_empty() {
                        return ErrorDialog::spawn(
                            &app_data.decrypt_gui.window,
                            "Invalid capture",
                            &format!("\"{}\" doesn't contain any valid handshake", file_path)
                        );
                    }

                    app_data.decrypt_gui.target_model.clear();

                    for (bssid, essid) in handshakes.iter() {
                        app_data.decrypt_gui.target_model.insert_with_values(None, &[(0, &bssid), (1, &essid)]);
                    }

                    app_data.decrypt_gui.target_view.set_active(if !handshakes.is_empty() { Some(0) } else { None });

                    app_data.decrypt_gui.handshake_entry.set_text(file_path);

                    update_decrypt_button_status(app_data);
                }
            }));
        }),
    );
}

fn connect_stack_notify(app_data: Rc<AppData>) {
    app_data
        .decrypt_gui
        .stack
        .connect_visible_child_notify(clone!(@strong app_data => move |_| {
            update_decrypt_button_status(app_data.clone());
        }));
}

fn connect_wordlist_button(app_data: Rc<AppData>) {
    app_data.decrypt_gui.wordlist_but.connect_clicked(
        clone!(@strong app_data => move |_| {
            let file_chooser_dialog = FileChooserDialog::new(
                Some("Select Wordlist"),
                Some(&app_data.decrypt_gui.window),
                FileChooserAction::Open,
                &[("Open", ResponseType::Accept)],
            );

            file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                this.close();

                if response == ResponseType::Accept {
                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };
                    app_data.decrypt_gui.wordlist_entry.set_text(gio_file.path().unwrap().to_str().unwrap());

                    update_decrypt_button_status(app_data);
                }
            }));
        }),
    );
}

fn connect_bruteforce_buttons(app_data: Rc<AppData>) {
    app_data
        .decrypt_gui
        .lowercase_but
        .connect_toggled(clone!(@strong app_data => move |_| {
            update_decrypt_button_status(app_data.clone());
        }));

    app_data
        .decrypt_gui
        .uppercase_but
        .connect_toggled(clone!(@strong app_data => move |_| {
            update_decrypt_button_status(app_data.clone());
        }));

    app_data
        .decrypt_gui
        .numbers_but
        .connect_toggled(clone!(@strong app_data => move |_| {
            update_decrypt_button_status(app_data.clone());
        }));

    app_data
        .decrypt_gui
        .symbols_but
        .connect_toggled(clone!(@strong app_data => move |_| {
            update_decrypt_button_status(app_data.clone());
        }));
}

fn connect_decrypt_button(app_data: Rc<AppData>) {
    app_data
        .decrypt_gui
        .decrypt_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            let handshake_entry = app_data.decrypt_gui.handshake_entry.text();
            let wordlist_entry = app_data.decrypt_gui.wordlist_entry.text();

            let low = app_data.decrypt_gui.lowercase_but.is_active();
            let up = app_data.decrypt_gui.uppercase_but.is_active();
            let num = app_data.decrypt_gui.numbers_but.is_active();
            let sym = app_data.decrypt_gui.symbols_but.is_active();

            let iter = match app_data.decrypt_gui.target_view.active_iter() {
                Some(iter) => iter,
                None => return,
            };
            let bssid = list_store_get!(app_data.decrypt_gui.target_model, &iter, 0, String);
            let essid = list_store_get!(app_data.decrypt_gui.target_model, &iter, 1, String);

            let stack = app_data.decrypt_gui.stack.visible_child_name().unwrap();

            if stack == "bruteforce" && !backend::has_dependency("crunch") {
                let err_msg = "\"crunch\" is not installed on your system, could not generate a wordlist from a charset";
                return ErrorDialog::spawn(&app_data.decrypt_gui.window, "Failed to run decryption", err_msg);
            }

            if stack == "dictionary" {
                if let Err(e) = backend::run_decrypt_wordlist_process(
                    &handshake_entry,
                    &bssid,
                    &essid,
                    &wordlist_entry
                ) {
                    return ErrorDialog::spawn(&app_data.decrypt_gui.window, "Failed to run decryption", &e.to_string());
                }
            } else if stack == "bruteforce" {
                if let Err(e) = backend::run_decrypt_bruteforce_process(
                    &handshake_entry,
                    &bssid,
                    &essid,
                    low,
                    up,
                    num,
                    sym
                ) {
                    return ErrorDialog::spawn(&app_data.decrypt_gui.window, "Failed to run decryption", &e.to_string());
                }
            }

            app_data.decrypt_gui.window.close();
        }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_controller(app_data.clone());

    connect_handshake_button(app_data.clone());
    connect_stack_notify(app_data.clone());
    connect_wordlist_button(app_data.clone());
    connect_bruteforce_buttons(app_data.clone());
    connect_decrypt_button(app_data);
}
