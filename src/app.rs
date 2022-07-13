pub struct MainApp {
  custom3d: crate::widgets::Custom3d,
}

impl MainApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    Self {
      custom3d: crate::widgets::Custom3d::new(cc),
    }
  }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
      egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
          ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
              frame.quit();
            }
          });
        });
      });

      egui::CentralPanel::default().show(ctx, |ui| {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
          self.custom3d.custom_painting(ui);
        });
      });

      egui::SidePanel::right("side_panel").show(ctx, |ui| {
        // TODO Side Panel
      }); 
    }
}