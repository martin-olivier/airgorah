use gtk4::prelude::*;
use gtk4::{Application, Settings};

mod backend;
mod error;
mod frontend;
mod globals;
mod types;

fn main() {
    env_logger::init();

    gtk4::init().expect("Could not initialize gtk4");

    let application = Application::builder()
        .application_id(globals::APP_ID)
        .build();

    let settings = Settings::default().unwrap();
    settings.set_gtk_icon_theme_name(Some("Adwaita"));

    application.connect_activate(frontend::build_ui);
    application.run();
}
