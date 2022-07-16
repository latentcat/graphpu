
fn main() {
    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        renderer: eframe::Renderer::Wgpu,
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    eframe::run_native(
        "graphpu",
        native_options, 
        Box::new(|cc| Box::new(graphpu::MainApp::new(cc)))
    );
}
