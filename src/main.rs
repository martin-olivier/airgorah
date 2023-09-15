use gtk4::prelude::*;
use gtk4::Application;

mod backend;
mod error;
mod frontend;
mod globals;
mod types;

fn main() {
    env_logger::init();

    let application = Application::builder()
        .application_id(globals::APP_ID)
        .build();

    application.connect_activate(frontend::build_ui);
    application.run();
}
