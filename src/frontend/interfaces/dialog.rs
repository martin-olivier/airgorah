use gtk4::prelude::*;
use gtk4::*;

pub struct ErrorDialog;

impl ErrorDialog {
    pub fn spawn(parent: &impl IsA<Window>, title: &str, content: &str, terminate: bool) {
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
            if terminate {
                std::process::exit(1);
            }
        });
        dialog.connect_close(move |_| {
            if terminate {
                std::process::exit(1);
            }
        });
        dialog.show();
    }
}

pub struct InfoDialog;

impl InfoDialog {
    pub fn spawn(parent: &impl IsA<Window>, title: &str, content: &str) {
        let dialog = MessageDialog::builder()
            .text(title)
            .secondary_text(content)
            .decorated(true)
            .message_type(MessageType::Info)
            .buttons(ButtonsType::Ok)
            .modal(true)
            .transient_for(parent)
            .build();

        dialog.connect_response(|this, _| {
            this.close();
        });
        dialog.show();
    }
}
