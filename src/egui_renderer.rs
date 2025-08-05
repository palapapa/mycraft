use egui::*;
use crate::egui_state::*;

pub trait EguiRenderer {
    fn render_ui(&mut self, egui_context: &Context, ui_state: &mut dyn EguiState);
}

/// The default implementation of [`EguiRenderer`].
#[derive(Default)]
pub struct DefaultEguiRenderer;

impl EguiRenderer for DefaultEguiRenderer {
    fn render_ui(&mut self, egui_context: &Context, ui_state: &mut dyn EguiState) {
        if egui_context.input(|input_state| input_state.key_pressed(Key::Backtick)) {
            ui_state.set_egui_enabled(!ui_state.is_egui_enabled());
        }
        if !ui_state.is_egui_enabled() {
            return;
        }
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
