use super::AppView;

pub struct MenuBarView;

impl Default for MenuBarView {
    fn default() -> Self {
        Self {  }
    }
}

impl AppView for MenuBarView {
    fn show(self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menubar_view").show(ctx, |ui| {
            egui::menu::bar(ui, |_| {
                // TODO: Menu Bar
            });
        });
    }
}
