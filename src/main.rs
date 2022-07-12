fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "graphpu",
        native_options, 
        Box::new(|cc| Box::new(graphpu::MainApp::new(cc)))
    );
}
