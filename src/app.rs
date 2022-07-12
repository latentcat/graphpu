pub struct MainApp;

impl Default for MainApp {
  fn default() -> Self {
      Self {}
  }
}

impl MainApp {
  pub fn new(_: &eframe::CreationContext<'_>) -> Self {
    Default::default()
  }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
      egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
          // TODO Menu Bar
        });
      });

      egui::CentralPanel::default().show(ctx, |ui| {
          
      });

      egui::SidePanel::right("side_panel").show(ctx, |ui| {
        // TODO Side Panel
      }); 
    }
}