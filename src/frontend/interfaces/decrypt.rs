use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::process::Stdio;
use std::rc::Rc;
use std::time::Duration;
use std::process::Command;

use super::dialog::*;
use crate::backend;
use crate::types::*;

pub struct DecryptWindow;

impl DecryptWindow {
    pub fn spawn(capture_file: Option<String>) {
        let window = Rc::new(
            Window::builder()
                .title("Decrypt Handshake")
                .default_width(500)
                .default_height(200)
                .resizable(false)
                .modal(true)
                .build(),
        );

        //

        let handshake_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: handshake.cap")
                .hexpand(true)
                .editable(false)
                .build(),
        );

        if let Some(path) = capture_file {
            handshake_entry.set_text(&path);
        }

        let handshake_but = Button::from_icon_name("edit-find-symbolic");

        let handshake_frame = Frame::new(Some("Handshake"));

        let handshake_box = Box::new(Orientation::Horizontal, 4);
        handshake_box.set_margin_start(4);
        handshake_box.set_margin_end(4);
        handshake_box.set_margin_bottom(4);
        handshake_box.append(handshake_entry.as_ref());
        handshake_box.append(&handshake_but);

        handshake_frame.set_child(Some(&handshake_box));

        //

        let wordlist_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: rockyou.txt")
                .hexpand(true)
                .editable(false)
                .build(),
        );

        let wordlist_but = Button::from_icon_name("edit-find-symbolic");

        let wordlist_frame = Frame::new(Some("Wordlist"));

        let wordlist_box = Box::new(Orientation::Horizontal, 4);
        wordlist_box.set_margin_start(4);
        wordlist_box.set_margin_end(4);
        wordlist_box.set_margin_bottom(4);
        wordlist_box.append(wordlist_entry.as_ref());
        wordlist_box.append(&wordlist_but);

        wordlist_frame.set_child(Some(&wordlist_box));

        //

        let decrypt_but = Rc::new(Button::with_label("Start Decryption"));
        decrypt_but.set_sensitive(false);

        //

        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.set_margin_top(10);
        vbox.set_margin_end(10);
        vbox.set_margin_start(10);
        vbox.set_margin_bottom(10);

        vbox.append(&handshake_frame);
        vbox.append(&wordlist_frame);
        vbox.append(&*decrypt_but);

        window.set_child(Some(&vbox));
        window.show();

        // Callbacks

        handshake_but.connect_clicked(
            clone!(@strong window, @strong decrypt_but, @strong handshake_entry, @strong wordlist_entry => move |_| {
                let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                    Some("Load Capture"),
                    Some(window.as_ref()),
                    FileChooserAction::Open,
                    &[("Open", ResponseType::Accept)],
                ));

                file_chooser_dialog.run_async(clone!(@strong decrypt_but, @strong handshake_entry, @strong wordlist_entry => move |this, response| {
                    if response == ResponseType::Accept {
                        let gio_file = match this.file() {
                            Some(file) => file,
                            None => return,
                        };
                        handshake_entry.set_text(gio_file.path().unwrap().to_str().unwrap());
                        if wordlist_entry.text_length() > 0 && handshake_entry.text_length() > 0 {
                            decrypt_but.set_sensitive(true);
                        }
                    }
                    this.close();
                }));
            }),
        );

        wordlist_but.connect_clicked(
            clone!(@strong window, @strong decrypt_but, @strong handshake_entry, @strong wordlist_entry => move |_| {
                let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                    Some("Load Capture"),
                    Some(window.as_ref()),
                    FileChooserAction::Open,
                    &[("Open", ResponseType::Accept)],
                ));

                file_chooser_dialog.run_async(clone!(@strong decrypt_but, @strong handshake_entry, @strong wordlist_entry => move |this, response| {
                    if response == ResponseType::Accept {
                        let gio_file = match this.file() {
                            Some(file) => file,
                            None => return,
                        };
                        wordlist_entry.set_text(gio_file.path().unwrap().to_str().unwrap());
                        if wordlist_entry.text_length() > 0 && handshake_entry.text_length() > 0 {
                            decrypt_but.set_sensitive(true);
                        }
                    }
                    this.close();
                }));
            }),
        );

        decrypt_but.connect_clicked(clone!(@strong window, @strong handshake_entry, @strong wordlist_entry => move |_| {
            Command::new("sh")
                .stdin(Stdio::piped())
                .args(["-c", &format!("gnome-terminal --hide-menubar --title \"Handshake Decryption\" -- bash -c \"aircrack-ng '{}' -w '{}' ; exec bash\"", handshake_entry.text().as_str(), wordlist_entry.text().as_str())])
                .output().ok();

            window.close();
        }));
    }
}
