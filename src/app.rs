pub struct MainApp {
  boids: crate::widgets::Boids,
}

impl MainApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    Self {
      boids: crate::widgets::Boids::new(cc),
    }
  }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
      egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |_| {
          // TODO Menu Bar
        });
      });

      egui::SidePanel::right("side_panel").show(ctx, |_| {
        // TODO Side Panel
      });

      egui::TopBottomPanel::bottom("detail_view").show(ctx, |ui| {
        egui::menu::bar(ui, |_| {
          // TODO Menu Bar
        });
      });

      egui::CentralPanel::default()
        .frame(egui::Frame::canvas(&ctx.style()))
        .show(ctx, |ui| {
          self.boids.custom_painting(ui);
      });
    }
}