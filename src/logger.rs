use env_logger::{Builder, Target};
use log::LevelFilter;

pub fn initialize() {
    Builder::new()
        .filter(Some("airgorah"), LevelFilter::Debug)
        .target(Target::Stdout)
        .init();

    log::debug!("logger initialized");
}
