use gtk4::prelude::*;
use gtk4::{ButtonsType, MessageDialog, MessageType, Window};

pub struct ErrorDialog;

impl ErrorDialog {
    pub fn spawn(parent: Option<&Window>, title: &str, content: &str) {
        let dialog = MessageDialog::builder()
            .text(title)
            .secondary_text(content)
            .decorated(true)
            .message_type(MessageType::Error)
            .buttons(ButtonsType::Close)
            .modal(true)
            .build();

        dialog.set_transient_for(parent);
        dialog.show();
        dialog.connect_response(|this, _| {
            this.close();
        });
    }
}

pub struct InfoDialog;

impl InfoDialog {
    pub fn spawn(parent: Option<&Window>, title: &str, content: &str) {
        let dialog = MessageDialog::builder()
            .text(title)
            .secondary_text(content)
            .decorated(true)
            .message_type(MessageType::Info)
            .buttons(ButtonsType::Ok)
            .modal(true)
            .build();

        dialog.set_transient_for(parent);
        dialog.show();
        dialog.connect_response(|this, _| {
            this.close();
        });
    }
}