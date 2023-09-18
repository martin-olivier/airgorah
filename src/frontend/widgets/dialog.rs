use gtk4::prelude::*;
use gtk4::*;

pub struct ErrorDialog;

impl ErrorDialog {
    pub fn spawn(parent: &impl IsA<Window>, title: &str, content: &str) {
        let dialog = MessageDialog::builder()
            .text(title)
            .secondary_text(content)
            .decorated(true)
            .message_type(MessageType::Error)
            .buttons(ButtonsType::Close)
            .modal(true)
            .transient_for(parent)
            .build();

        dialog.connect_response(move |this, _| {
            this.close();
        });
        dialog.show();
    }
}

pub struct PanicDialog;

impl PanicDialog {
    pub fn spawn(parent: &impl IsA<Window>, message: &str) {
        let dialog = MessageDialog::builder()
            .text("Error")
            .secondary_text(message)
            .decorated(true)
            .message_type(MessageType::Error)
            .buttons(ButtonsType::Close)
            .modal(true)
            .transient_for(parent)
            .build();

        dialog.connect_response(move |this, _| {
            this.close();
            std::process::exit(1);
        });
        dialog.connect_close(move |_| {
            std::process::exit(1);
        });
        dialog.show();
    }
}

pub struct UpdateDialog;

impl UpdateDialog {
    pub fn spawn(parent: &impl IsA<Window>, version: &str, new_version: &str) {
        let title = format!("Update available ({} -> {})", version, new_version);
        let link = "https://github.com/martin-olivier/airgorah/releases/latest";

        let dialog = MessageDialog::builder()
            .text(title)
            .secondary_text(link)
            .decorated(true)
            .message_type(MessageType::Info)
            .modal(true)
            .transient_for(parent)
            .build();

        dialog.add_button("Close", ResponseType::Close);
        dialog.add_button("Copy Link", ResponseType::Other(42));

        dialog.connect_response(|this, response| {
            if response == ResponseType::Other(42) {
                if let Some(display) = gdk::Display::default() {
                    display.clipboard().set_text(link);
                }
            }
            this.close();
        });
        dialog.show();
    }
}
