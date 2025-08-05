pub trait EguiState {
    /// `true` if the UI should be shown; `false` if it should be hidden.
    fn is_egui_enabled(&self) -> bool;

    /// Sets if the UI should be shown.
    fn set_egui_enabled(&mut self, enabled: bool);
}

/// The default implementation of [`EguiState`].
pub struct DefaultEguiState {
    is_egui_enabled: bool
}

impl DefaultEguiState {
    pub const fn new() -> Self {
        Self {
            is_egui_enabled: true
        }
    }
}

impl EguiState for DefaultEguiState {
    fn is_egui_enabled(&self) -> bool {
        self.is_egui_enabled
    }

    fn set_egui_enabled(&mut self, enabled: bool) {
        self.is_egui_enabled = enabled;
    }
}
