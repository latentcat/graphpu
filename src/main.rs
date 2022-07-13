
fn main() {
    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        // #[cfg(feature = "wgpu")]
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "GraphPU",
        native_options, 
        Box::new(|cc| Box::new(graphpu::MainApp::new(cc)))
    );
}
