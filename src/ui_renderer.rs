use egui::*;

pub trait UiRenderer {
    fn draw_ui(&mut self, egui_context: &Context);
}

#[derive(Default)]
pub struct DefaultUiRenderer;

impl UiRenderer for DefaultUiRenderer {
    fn draw_ui(&mut self, egui_context: &Context) {
        Window::new("Hello, World!")
            .vscroll(true)
            .show(
                egui_context,
                |ui| {
                    ui.label("Hello, Label!");
                }
            );
    }
}
