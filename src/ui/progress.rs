use gtk4::prelude::*;
use gtk4::{ProgressBar, Window};
use std::rc::Rc;
use std::time::Duration;

pub struct ProgressWindow {
    pub window: Rc<Window>,
}

impl ProgressWindow {
    pub fn spawn<F: FnMut() + 'static>(timeout: u64, mut callback: F) -> Self {
        let window = Rc::new(
            Window::builder()
                .title("")
                .default_width(250)
                .default_height(30)
                .resizable(false)
                .modal(true)
                .build(),
        );
        let bar = Rc::new(ProgressBar::new());
        bar.set_text(Some("Scanning nearby networks..."));
        bar.set_show_text(true);
        let refresh_rate = (timeout * 1000) / 100;

        let win_ref = window.clone();
        let bar_ref = bar.clone();
        glib::timeout_add_local(Duration::from_millis(refresh_rate), move || {
            bar_ref.set_fraction(bar_ref.fraction() + 0.01);
            if bar_ref.fraction() >= 1.0 {
                win_ref.close();
                callback();
                return glib::Continue(false);
            }
            glib::Continue(true)
        });

        bar.set_margin_top(10);
        bar.set_margin_end(10);
        bar.set_margin_start(10);
        bar.set_margin_bottom(10);

        window.set_child(Some(&*bar));
        Self { window }
    }
}
