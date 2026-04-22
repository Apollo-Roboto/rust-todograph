#![allow(unused)]

use std::fs::OpenOptions;

use tg_core::{Editor, LOGGER, SaveData};

use log::{debug, error, info, warn};

const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const APPLICATION_IS_RELEASE: bool = !cfg!(debug_assertions);

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        error!("Missing path argument");
        return;
    };

    let data = load(path);

    print_details(&data);
}

fn print_details(data: &SaveData) {
    println!("Version: {}", data.metadata.version);
    println!(
        "Date: {}",
        data.metadata
            .save_date
            .map(|d| d.to_string())
            .unwrap_or("Unknown".to_string())
    );
    println!(
        "Tasks: {} / {}",
        data.tasks.iter().filter(|t| t.state.is_done()).count(),
        data.tasks.len()
    );
}

fn load(path: &String) -> SaveData {
    let start = std::time::Instant::now();
    debug!("Loading {path:?}");
    let file = OpenOptions::new()
        .write(false)
        .read(true)
        .open(path.clone())
        .map_err(|e| e.to_string())
        .unwrap();

    let data: SaveData = serde_json::from_reader(file)
        .map_err(|e| e.to_string())
        .unwrap();

    let end = std::time::Instant::now();
    let time_to_load = end - start;

    info!("Loaded {path:?} [{time_to_load:?}]");

    data
}
