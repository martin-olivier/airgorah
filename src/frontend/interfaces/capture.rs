use glib::clone;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use std::time::Duration;

use super::DecryptWindow;
use super::dialog::*;
use crate::backend;
use crate::types::*;

pub struct CaptureWindow;

impl CaptureWindow {
    pub fn spawn(parent: &impl IsA<Window>, ap: AP) {
        let window = Rc::new(
            Window::builder()
                .title(&format!("Capture Handshake on \"{}\"", ap.essid))
                .default_width(300)
                .default_height(140)
                .resizable(false)
                .modal(true)
                .build(),
        );

        window.set_transient_for(Some(parent));

        let passive_but = CheckButton::builder()
            .label("Passive mode")
            .tooltip_text("Wait for a client to connect to the access point")
            .build();
        let deauth_but = CheckButton::builder()
            .label("Deauth mode")
            .tooltip_text("Force clients to disconnect and reconnect to capture the handshake")
            .build();

        passive_but.set_active(true);
        deauth_but.set_group(Some(&passive_but));

        passive_but.set_margin_start(15);
        passive_but.set_margin_top(15);

        deauth_but.set_margin_start(15);
        deauth_but.set_margin_bottom(15);

        let but_box = Box::new(Orientation::Vertical, 10);
        but_box.append(&passive_but);
        but_box.append(&deauth_but);

        let frame = Frame::new(None);
        frame.set_child(Some(&but_box));

        let path_entry = Rc::new(
            Entry::builder()
                .placeholder_text("ex: /root/handshake.cap")
                .hexpand(true)
                .editable(false)
                .build(),
        );

        let path_but = Button::from_icon_name("edit-find-symbolic");

        let path_frame = Frame::new(Some("Save to"));

        let path_box = Box::new(Orientation::Horizontal, 4);
        path_box.set_margin_start(4);
        path_box.set_margin_end(4);
        path_box.set_margin_bottom(4);
        path_box.append(path_entry.as_ref());
        path_box.append(&path_but);

        path_frame.set_child(Some(&path_box));

        let capture_but = Rc::new(Button::with_label("Start Capture"));
        capture_but.set_sensitive(false);

        let spinner = Rc::new(Spinner::new());
        spinner.set_spinning(true);
        spinner.set_sensitive(false);
        spinner.hide();

        let main_box = Box::new(Orientation::Vertical, 10);
        main_box.append(&frame);
        main_box.append(&path_frame);
        main_box.append(&*capture_but);
        main_box.append(&*spinner);

        main_box.set_margin_bottom(10);
        main_box.set_margin_end(10);
        main_box.set_margin_start(10);
        main_box.set_margin_top(10);

        window.set_child(Some(&main_box));
        window.show();

        // Callbacks

        let network_name = ap.essid.clone();
        path_but.connect_clicked(
            clone!(@strong capture_but, @strong network_name, @strong window, @strong path_entry => move |_| {
                let file_chooser_dialog = Rc::new(FileChooserDialog::new(
                    Some("Save Capture"),
                    Some(window.as_ref()),
                    FileChooserAction::Save,
                    &[("Save", ResponseType::Accept)],
                ));

                file_chooser_dialog.set_current_name(&(network_name.clone() + ".cap"));
                file_chooser_dialog.run_async(clone!(@strong capture_but, @strong path_entry => move |this, response| {
                    if response == ResponseType::Accept {
                        let gio_file = match this.file() {
                            Some(file) => file,
                            None => return,
                        };
                        path_entry.set_text(gio_file.path().unwrap().to_str().unwrap());
                        capture_but.set_sensitive(true);
                    }
                    this.close();
                }));
            }),
        );

        capture_but.connect_clicked(clone!(@strong window, @strong passive_but, @strong deauth_but, @strong path_entry, @strong path_but => move |this| {
            if backend::is_capture_process() {
                spinner.hide();
                spinner.stop();
                passive_but.set_sensitive(true);
                deauth_but.set_sensitive(true);
                path_but.set_sensitive(true);
                this.set_label("Start Capture");

                backend::stop_capture_process();
                if deauth_but.is_active() {
                    backend::stop_deauth_attack(&ap.bssid);
                }
            } else {
                spinner.show();
                spinner.start();
                passive_but.set_sensitive(false);
                deauth_but.set_sensitive(false);
                path_but.set_sensitive(false);
                this.set_label("Stop Capture");

                backend::set_capture_process(ap.clone()).unwrap();
                if deauth_but.is_active() {
                    backend::launch_deauth_attack(ap.clone(), None).unwrap();
                }

                glib::timeout_add_local(Duration::from_secs(1), clone!(@strong ap, @strong window, @strong deauth_but, @strong path_entry => move || {
                    if !backend::is_capture_process() {
                        return glib::Continue(false);
                    }

                    if backend::has_handshake().unwrap() {
                        let path = path_entry.text();
                        backend::save_capture(&path);
                        
                        YesNoDialog::spawn(window.as_ref(), "Handshake Captured", "Handshake has been captured!\nWould you like to decrypt the password now?", move |this, response| {
                            if response == ResponseType::Yes {
                                DecryptWindow::spawn(Some(path.to_string()));
                            }
                            this.close();
                        });

                        backend::stop_capture_process();
                        if deauth_but.is_active() {
                            backend::stop_deauth_attack(&ap.bssid);
                        }
                        window.close();
                        return glib::Continue(false);
                    }
                    glib::Continue(true)
                }));
            }
        }));
    }
}
