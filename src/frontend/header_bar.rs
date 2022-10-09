use gtk4::prelude::*;
use gtk4::*;

fn build_about_button() -> Button {
    let about_button = Button::builder().icon_name("dialog-information").build();
    about_button.connect_clicked(|_| {
        let about = AboutDialog::builder()
            .program_name("Airgorah")
            .version("0.1 beta")
            .authors(vec!["Martin OLIVIER (martin.olivier@live.fr)".to_string()])
            .copyright("Copyright (c) Martin OLIVIER")
            .license_type(License::MitX11)
            .comments("A GUI around aircrack-ng suite tools")
            .logo_icon_name("network-wireless-hotspot")
            .website_label("https://github.com/martin-olivier/airgorah")
            .build();
        about.show();
    });

    about_button
}

pub fn build_header_bar(window: &ApplicationWindow) {
    let header_bar = HeaderBar::new();
    header_bar.set_show_title_buttons(true);
    
    window.set_titlebar(Some(&header_bar));

    let about_but = build_about_button();
    header_bar.pack_start(&about_but);
}