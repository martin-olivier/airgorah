use crate::frontend::interfaces::*;
use glib::clone;
use gtk4::prelude::*;
use std::rc::Rc;

pub fn connect_capture_button(app_data: Rc<AppData>) {
    app_data
        .capture_but
        .connect_clicked(clone!(@strong app_data => move |_| {
            InfoDialog::spawn(
                &app_data.main_window,
                "Comming Soon",
                "This feature will be available in a future version",
            );
        }));
}
