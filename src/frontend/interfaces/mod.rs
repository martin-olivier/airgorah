pub mod app;
pub mod interface;
pub mod deauth;
pub mod decrypt;

pub use app::AppGui;
pub use interface::InterfaceGui;
pub use deauth::DeauthGui;
pub use decrypt::DecryptGui;

pub struct AppData {
    pub app_gui: AppGui,
    pub interface_gui: InterfaceGui,
    pub deauth_gui: DeauthGui,
    pub decrypt_gui: DecryptGui,
}

impl AppData {
    pub fn new(app: &gtk4::Application) -> Self {
        let app_gui = AppGui::new(app);
        let interface_gui = InterfaceGui::new(&app_gui.window);
        let deauth_gui = DeauthGui::new(&app_gui.window);
        let decrypt_gui = DecryptGui::new(&app_gui.window);

        Self {
            app_gui,
            interface_gui,
            deauth_gui,
            decrypt_gui,
        }
    }
}