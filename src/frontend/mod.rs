mod connections;
mod interfaces;
mod widgets;

use crate::backend;
use interfaces::*;
use widgets::*;

use gtk4::*;
use std::rc::Rc;

pub fn build_ui(app: &Application) {
    let gui_data = Rc::new(AppData::new(app));

    if let Err(e) = backend::app_setup() {
        return ErrorDialog::spawn(&gui_data.app_gui.window, "Error", &e.to_string(), true);
    }

    connections::connect(gui_data);
}

#[macro_export]
macro_rules! list_store_get {
    ($storage:expr,$iter:expr,$pos:expr,$typ:ty) => {
        $storage.get_value($iter, $pos).get::<$typ>().unwrap()
    };
}
