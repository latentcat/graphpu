pub struct MainApp {
  custom3d: crate::widgets::Custom3d,
  boids: crate::widgets::Boids,
}

impl MainApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    Self {
      custom3d: crate::widgets::Custom3d::new(cc),
      boids: crate::widgets::Boids::new(cc),
    }
  }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
      egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
          // TODO Menu Bar
        });
      });

      egui::CentralPanel::default()
        .frame(egui::Frame::canvas(&ctx.style()))
        .show(ctx, |ui| {
          self.boids.custom_painting(ui);
      });

      egui::SidePanel::right("side_panel").show(ctx, |ui| {
        // TODO Side Panel
      }); 
    }
}