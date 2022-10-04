use gtk4::prelude::*;
use gtk4::{ProgressBar, Window};
use std::rc::Rc;
use std::time::Duration;

pub struct ProgressWindow {
    pub window: Rc<Window>,
}

impl ProgressWindow {
    pub fn spawn<F: Fn() + 'static>(timeout: u64, callback: F) -> Self {
        let window = Rc::new(
            Window::builder()
                .title("Scanning nearby networks...")
                .default_width(250)
                .default_height(50)
                .resizable(false)
                .modal(true)
                .build(),
        );
        let bar = Rc::new(ProgressBar::new());
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

        window.set_child(Some(&*bar));
        Self { window }
    }
}
