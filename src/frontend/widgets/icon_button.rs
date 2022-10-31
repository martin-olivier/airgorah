#![allow(unused)]

use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::*;

pub struct IconButton {
    pub handle: Button,
    image: Image,
}

impl IconButton {
    pub fn new(icon: &'static [u8]) -> Self {
        let pixbuf = Pixbuf::from_read(std::io::BufReader::new(icon)).unwrap();

        let image = Image::from_pixbuf(Some(&pixbuf));

        let handle = Button::builder().child(&image).build();

        Self { handle, image }
    }

    pub fn set_tooltip_text(&self, text: Option<&str>) {
        self.handle.set_tooltip_text(text)
    }

    pub fn set_sensitive(&self, sensitive: bool) {
        self.handle.set_sensitive(sensitive)
    }

    pub fn set_icon(&self, icon: &'static [u8]) {
        let pixbuf = Pixbuf::from_read(std::io::BufReader::new(icon)).unwrap();
        self.image.set_from_pixbuf(Some(&pixbuf))
    }

    pub fn connect_clicked<F: Fn(&Button) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.handle.connect_clicked(f)
    }

    pub fn set_margin_bottom(&self, margin_bottom: i32) {
        self.handle.set_margin_bottom(margin_bottom)
    }

    pub fn set_margin_end(&self, margin_end: i32) {
        self.handle.set_margin_end(margin_end)
    }

    pub fn set_margin_start(&self, margin_start: i32) {
        self.handle.set_margin_start(margin_start)
    }

    pub fn set_margin_top(&self, margin_top: i32) {
        self.handle.set_margin_top(margin_top)
    }
}
