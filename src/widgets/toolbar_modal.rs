use egui::{Color32, Context, Id, InnerResponse, Pos2, Shape, Ui};

pub struct ToolbarModal {
    wrapper_id: Id,
    inner_id: Id,
}

impl ToolbarModal {
    pub fn new(id_source: String) -> Self {
        let wrapper_id = id_source.clone() + "_wrapper";
        Self {
            wrapper_id: Id::new(wrapper_id),
            inner_id: Id::new(id_source),
        }
    }

    pub fn show_inside<R>(
        self,
        ui: &Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let max_rect = ui.max_rect();
        egui::Area::new(self.inner_id)
            .fixed_pos(ui.max_rect().min)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let interceptor_rect = max_rect;
                // ui.allocate_response(interceptor_rect.size(), egui::Sense::hover());
                ui.allocate_ui_at_rect(interceptor_rect, |ui| {
                    ui.painter().add(Shape::rect_filled(
                        interceptor_rect,
                        0.,
                        Color32::from_black_alpha(180),
                    ));
                    add_contents(ui)
                }).inner
            })
    }
}
