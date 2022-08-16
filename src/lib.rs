#[macro_use]
extern crate lazy_static;

mod app;
pub mod widgets;
pub mod components;
pub mod models;
pub mod utils;
pub mod bootstrap;

pub use app::MainApp;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
