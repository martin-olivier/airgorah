use crate::frontend::interfaces::*;

use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::process::Stdio;
use std::rc::Rc;
use std::process::Command;

fn connect_handshake_button(app_data: Rc<AppData>) {
    app_data.decrypt_gui.handshake_but.connect_clicked(
        clone!(@strong app_data => move |_| {

            let file_chooser_dialog = FileChooserDialog::new(
                Some("Load Capture"),
                Some(&app_data.decrypt_gui.window),
                FileChooserAction::Open,
                &[("Open", ResponseType::Accept)],
            );

            file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                if response == ResponseType::Accept {
                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };
                    app_data.decrypt_gui.handshake_entry.set_text(gio_file.path().unwrap().to_str().unwrap());
                    if app_data.decrypt_gui.wordlist_entry.text_length() > 0 && app_data.decrypt_gui.handshake_entry.text_length() > 0 {
                        app_data.decrypt_gui.decrypt_but.set_sensitive(true);
                    }
                }
                this.close();
            }));
        }),
    );
}

fn connect_wordlist_button(app_data: Rc<AppData>) {
    app_data.decrypt_gui.wordlist_but.connect_clicked(
        clone!(@strong app_data => move |_| {
            let file_chooser_dialog = FileChooserDialog::new(
                Some("Load Capture"),
                Some(&app_data.decrypt_gui.window),
                FileChooserAction::Open,
                &[("Open", ResponseType::Accept)],
            );

            file_chooser_dialog.run_async(clone!(@strong app_data => move |this, response| {
                if response == ResponseType::Accept {
                    let gio_file = match this.file() {
                        Some(file) => file,
                        None => return,
                    };
                    app_data.decrypt_gui.wordlist_entry.set_text(gio_file.path().unwrap().to_str().unwrap());
                    if app_data.decrypt_gui.wordlist_entry.text_length() > 0 && app_data.decrypt_gui.handshake_entry.text_length() > 0 {
                        app_data.decrypt_gui.decrypt_but.set_sensitive(true);
                    }
                }
                this.close();
            }));
        }),
    );
}

fn connect_decrypt_button(app_data: Rc<AppData>) {
    app_data.decrypt_gui.decrypt_but.connect_clicked(clone!(@strong app_data => move |_| {
        Command::new("sh")
            .stdin(Stdio::piped())
            .args(["-c", &format!("gnome-terminal --hide-menubar --title \"Handshake Decryption\" -- bash -c \"aircrack-ng '{}' -w '{}' ; exec bash\"", app_data.decrypt_gui.handshake_entry.text().as_str(), app_data.decrypt_gui.wordlist_entry.text().as_str())])
            .output().ok();

        app_data.decrypt_gui.window.close();
    }));
}

pub fn connect(app_data: Rc<AppData>) {
    connect_handshake_button(app_data.clone());
    connect_wordlist_button(app_data.clone());
    connect_decrypt_button(app_data);
}
