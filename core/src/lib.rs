pub mod commands;
pub mod editor;
pub mod graph;
mod models;

pub use editor::Editor;
pub use graph::TaskGraph;
pub use models::*;

const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const APPLICATION_IS_RELEASE: bool = !cfg!(debug_assertions);
