#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use graphpu::bootstrap::{start_frame, ConfigBuilder};

#[tokio::main]
async fn main() {
    start_frame(ConfigBuilder::default().build());
}
