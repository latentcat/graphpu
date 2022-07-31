use graphpu::{APP_VERSION, MainApp};

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(1000.0, 600.0)),
        min_window_size: Some(egui::Vec2::new(800.0, 500.0)),
        drag_and_drop_support: true,
        renderer: eframe::Renderer::Wgpu,
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    eframe::run_native(
        &*format!("GraphPU - Dev Demo - {}", APP_VERSION),
        native_options,
        Box::new(|cc| Box::new(MainApp::new(cc)))
    );
}
