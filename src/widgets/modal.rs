use egui::{Color32, Context, Id, InnerResponse, Pos2, Shape, Ui};

pub struct Modal {
    wrapper_id: Id,
    inner_id: Id,
}

impl Modal {
    pub fn new(id_source: String) -> Self {
        let wrapper_id = id_source.clone() + "_wrapper";
        Self {
            wrapper_id: Id::new(wrapper_id),
            inner_id: Id::new(id_source),
        }
    }

    pub fn show<R>(
        self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        egui::Area::new(self.wrapper_id)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let interceptor_rect = ui.ctx().input().screen_rect();
                ui.allocate_response(interceptor_rect.size(), egui::Sense::hover());
                ui.allocate_ui_at_rect(interceptor_rect, |ui| {
                    ui.painter().add(Shape::rect_filled(
                        interceptor_rect,
                        0.,
                        Color32::from_black_alpha(180),
                    ));

                    egui::Area::new(self.inner_id)
                        .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                        .order(egui::Order::Foreground)
                        .show(ui.ctx(), |ui| {
                            crate::widgets::frames::window_frame(&ui.style())
                                .show(ui, |ui| add_contents(ui))
                                .inner
                        })
                        .inner
                })
                .inner
            })
    }
}
