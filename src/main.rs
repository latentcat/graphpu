#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use graphpu::bootstrap::{start_frame, ConfigBuilder};

fn main() {
    start_frame(ConfigBuilder::default().build());
}
