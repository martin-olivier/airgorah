mod app;
mod deauth;
mod decrypt;
mod interface;
mod scan;
mod settings;

use crate::frontend::interfaces::AppData;
use std::rc::Rc;

pub fn connect(app: &gtk4::Application, app_data: Rc<AppData>) {
    app::connect(app, app_data.clone());
    scan::connect(app_data.clone());
    interface::connect(app_data.clone());
    deauth::connect(app_data.clone());
    decrypt::connect(app_data.clone());
    settings::connect(app_data);
}
