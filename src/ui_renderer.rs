use egui::*;
use crate::ui_state::*;

pub trait UiRenderer {
    fn render_ui(&mut self, egui_context: &Context, ui_state: &mut dyn UiState);
}

#[derive(Default)]
pub struct DefaultUiRenderer;

impl UiRenderer for DefaultUiRenderer {
    fn render_ui(&mut self, egui_context: &Context, ui_state: &mut dyn UiState) {
        if egui_context.input(|input_state| input_state.key_pressed(Key::Backtick)) {
            ui_state.set_ui_enabled(!ui_state.is_ui_enabled());
        }
        if !ui_state.is_ui_enabled() {
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
