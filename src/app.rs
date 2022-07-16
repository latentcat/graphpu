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
              // TODO: Menu
            });
        });

        egui::SidePanel::right("inspector")
          .resizable(false)
          .show(ctx, |_| {
            // TODO: Inspector
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&ctx.style()))
            .show(ctx, |ui| {
                self.boids.custom_painting(ui);
            });

        egui::TopBottomPanel::bottom("detail").show(ctx, |_| {
          // TODO: Detail
        });
    }
}
