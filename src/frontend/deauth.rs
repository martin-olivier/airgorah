use gtk4::prelude::*;
use gtk4::*;
//use std::rc::Rc;

use crate::backend::AP;

pub struct DeauthWindow {
    pub window: Window,
}

impl DeauthWindow {
    pub fn new(parent: &impl IsA<Window>, ap: AP) -> Self {
        let window = Window::builder()
            .title(&format!("Deauth \"{}\"", ap.essid))
            .default_width(350)
            .default_height(70)
            .resizable(false)
            .modal(true)
            .build();

        window.set_transient_for(Some(parent));

        let all_cli_but = CheckButton::new();
        let sel_cli_but = CheckButton::new();

        sel_cli_but.set_group(Some(&all_cli_but));

        let all_cli_lab = Label::new(Some("Deauth all clients"));
        let sel_cli_lab = Label::new(Some("Deauth selected clients"));

        let all_cli_box = Box::new(Orientation::Horizontal, 10);
        let sel_cli_box = Box::new(Orientation::Horizontal, 10);

        all_cli_box.append(&all_cli_but);
        all_cli_box.append(&all_cli_lab);

        sel_cli_box.append(&sel_cli_but);
        sel_cli_box.append(&sel_cli_lab);

        let main_box = Box::new(Orientation::Vertical, 10);

        main_box.append(&all_cli_box);
        main_box.append(&sel_cli_box);

        window.set_child(Some(&main_box));
        window.show();

        /*let mut args = vec![];
        args.push("--bssid");
        args.push(&ap.bssid);
        args.push("--client");
        args.push(&cli.mac);

        backend::launch_deauth_process(&args);*/

        Self { window }
    }
}