use crate::frontend::interfaces::*;
use crate::list_store_get;
use crate::{backend, globals};
use glib::clone;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::prelude::*;
use gtk4::*;
use std::rc::Rc;
use std::time::Duration;

pub fn connect_about_button(app_data: Rc<AppData>) {
    app_data
        .about_button
        .connect_clicked(clone!(@strong app_data => move |_| {
        let ico = Pixbuf::from_read(std::io::BufReader::new(globals::APP_ICON)).unwrap();
        let des = "A WiFi auditing software that can perform deauth attacks and passwords cracking";

        AboutDialog::builder()
            .program_name("Airgorah")
            .version(globals::VERSION)
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("Copyright (c) Martin OLIVIER")
            .license_type(License::MitX11)
            .logo(&Picture::for_pixbuf(&ico).paintable().unwrap())
            .comments(des)
            .website_label("https://github.com/martin-olivier/airgorah")
            .transient_for(&app_data.main_window)
            .modal(true)
            .build()
            .show();
        }));
}

pub fn connect_update_button(app_data: Rc<AppData>) {

    globals::UPDATE_PROC.lock().unwrap().replace(backend::spawn_update_checker());

    glib::timeout_add_local(Duration::from_millis(1000), clone!(@strong app_data => move || {
        let mut updater = globals::UPDATE_PROC.lock().unwrap();

        if let Some(proc) = updater.as_mut() {
            if proc.is_finished() {
                if updater.take().unwrap().join().unwrap_or(false) == true {
                    app_data.update_button.show();
                    return glib::Continue(false);
                }
            }
        }
        return glib::Continue(true);
    }));

    app_data
        .update_button
        .connect_clicked(clone!(@strong app_data => move |_| {
            InfoDialog::spawn(&app_data.main_window, "Update available", "An update is available, you can download it on the following page:\n\nhttps://github.com/martin-olivier/airgorah/releases/latest");
        }));
}

pub fn connect_decrypt_button(app_data: Rc<AppData>) {
    app_data
        .decrypt_button
        .connect_clicked(clone!(@strong app_data => move |_| {
            DecryptWindow::spawn(None);
        }));
}
