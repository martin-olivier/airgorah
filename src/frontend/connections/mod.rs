mod deauth;
mod interface;
mod app;
mod scan;
mod decrypt;

use crate::frontend::interfaces::AppData;
use std::rc::Rc;

pub fn connect(app_data: Rc<AppData>) {
    app::connect(app_data.clone());
    scan::connect(app_data.clone());
    interface::connect(app_data.clone());
    deauth::connect(app_data.clone());
    decrypt::connect(app_data);
}