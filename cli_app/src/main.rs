#![allow(unused)]
use log::{info, warn};
use rust_firework_core::LOGGER;

const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const APPLICATION_IS_RELEASE: bool = !cfg!(debug_assertions);

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("Hello, world!");
}
