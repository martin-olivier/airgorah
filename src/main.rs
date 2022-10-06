use gtk4::prelude::*;
use gtk4::Application;

mod backend;
mod globals;
mod ui;

fn main() {
    sudo::escalate_if_needed().unwrap();
    let application = Application::builder()
        .application_id("com.martin-olivier.airgorah")
        .build();

    application.connect_activate(ui::build_ui);
    application.run();
}
