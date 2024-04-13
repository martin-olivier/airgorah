mod connections;
mod interfaces;
mod widgets;

use crate::backend;
use interfaces::*;
use widgets::*;

use gtk4::prelude::*;
use gtk4::*;

use std::rc::Rc;

pub fn build_ui(app: &Application) {
    let gui_data = Rc::new(AppData::new(app));

    gui_data.app_gui.window.show();

    if let Err(e) = backend::app_setup() {
        return PanicDialog::spawn(&gui_data.app_gui.window, &e.to_string());
    }

    gui_data.interface_gui.window.show();

    connections::connect(app, gui_data);
}

#[macro_export]
macro_rules! list_store_get {
    ($storage:expr,$iter:expr,$pos:expr,$typ:ty) => {
        $storage.get_value($iter, $pos).get::<$typ>().unwrap()
    };
}
