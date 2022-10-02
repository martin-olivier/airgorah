use gtk::prelude::*;
use gtk::Application;
use gtk4 as gtk;

mod backend;
mod globals;
mod ui;

fn main() {
    let application = Application::builder()
        .application_id("com.martin-olivier.airgorah")
        .build();

    application.connect_activate(ui::build_ui);
    application.run();
}
