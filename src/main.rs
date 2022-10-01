use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
use gtk4 as gtk;

mod backend;
mod interface;

fn main() {
    let application = Application::builder()
        .application_id("com.martin-olivier.airgorah")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Airgorah")
            .default_width(350)
            .default_height(70)
            .build();

        let boxx = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

        let button = Button::with_label("Click me!");
        button.connect_clicked(|_| {
            let about = gtk4::AboutDialog::new();
            about.show();
            eprintln!("Clicked! hihi");
        });

        boxx.append(&button);
        window.set_child(Some(&boxx));
        window.show();

        interface::interfaces_ui();
    });

    application.run();
}
