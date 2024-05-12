use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::{Application, Settings};

mod backend;
mod frontend;
mod globals;
mod types;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    gtk4::init().expect("Could not initialize gtk4");

    Settings::default()
        .unwrap()
        .set_gtk_icon_theme_name(Some("Adwaita"));

    let application = Application::builder()
        .application_id(globals::APP_ID)
        .flags(ApplicationFlags::NON_UNIQUE)
        .build();

    application.connect_activate(frontend::build_ui);
    application.run();
}
