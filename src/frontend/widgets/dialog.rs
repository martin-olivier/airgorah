#![allow(unused)]

use clipboard::{ClipboardContext, ClipboardProvider};
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

pub struct UpdateDialog;

impl UpdateDialog {
    pub fn spawn(parent: &impl IsA<Window>, version: &str, new_version: &str) {
        let link = "https://github.com/martin-olivier/airgorah/releases/latest";
        let title = format!("Update available ({} -> {})", version, new_version);
        let body = format!("A new version of Airgorah is available.\nYou can download it on the following page:\n\n{}", link);

        let dialog = MessageDialog::builder()
            .text(title)
            .secondary_text(body)
            .decorated(true)
            .message_type(MessageType::Info)
            .modal(true)
            .transient_for(parent)
            .build();

        dialog.add_button("Close", ResponseType::Close);
        dialog.add_button("Copy Link", ResponseType::Other(42));

        dialog.connect_response(|this, response| {
            if response == ResponseType::Other(42) {
                let clip: Result<ClipboardContext, _> = ClipboardProvider::new();

                if let Ok(mut clip) = clip {
                    clip.set_contents(link.to_string()).ok();
                }
            }
            this.close();
        });
        dialog.show();
    }
}
