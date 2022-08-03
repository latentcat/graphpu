mod app;
pub mod widgets;
pub mod components;
pub mod models;
pub mod utils;
pub mod bootstrap;
pub mod wlib;

pub use app::MainApp;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
