pub mod app;
pub mod deauth;
pub mod decrypt;
pub mod interface;
pub mod settings;

pub use app::AppGui;
pub use deauth::DeauthGui;
pub use decrypt::DecryptGui;
pub use interface::InterfaceGui;
pub use settings::SettingsGui;

pub struct AppData {
    pub app_gui: AppGui,
    pub interface_gui: InterfaceGui,
    pub deauth_gui: DeauthGui,
    pub decrypt_gui: DecryptGui,
    pub settings_gui: SettingsGui,
}

impl AppData {
    pub fn new(app: &gtk4::Application) -> Self {
        let app_gui = AppGui::new(app);
        let interface_gui = InterfaceGui::new(&app_gui.window);
        let deauth_gui = DeauthGui::new(&app_gui.window);
        let decrypt_gui = DecryptGui::new(&app_gui.window);
        let settings_gui = SettingsGui::new(&app_gui.window);

        Self {
            app_gui,
            interface_gui,
            deauth_gui,
            decrypt_gui,
            settings_gui,
        }
    }
}
