pub trait UiState {
    /// `true` if the UI should be shown; `false` if it should be hidden.
    fn is_ui_enabled(&self) -> bool;

    /// Sets if the UI should be shown.
    fn set_ui_enabled(&mut self, enabled: bool);
}

pub struct DefaultUiState {
    is_ui_enabled: bool
}

impl DefaultUiState {
    pub const fn new() -> Self {
        Self {
            is_ui_enabled: true
        }
    }
}

impl UiState for DefaultUiState {
    fn is_ui_enabled(&self) -> bool {
        self.is_ui_enabled
    }

    fn set_ui_enabled(&mut self, enabled: bool) {
        self.is_ui_enabled = enabled;
    }
}
