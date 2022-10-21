use gtk4::prelude::*;
use gtk4::Application;

mod frontend;
mod backend;
mod globals;
mod types;
mod error;

fn main() {
    let application = Application::builder()
        .application_id("com.martin-olivier.airgorah")
        .build();

    application.connect_activate(frontend::build_ui);
    application.run();
}
