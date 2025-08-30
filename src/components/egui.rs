use bevy_ecs::component::*;
use crate::egui_renderer::*;

#[derive(Component)]
pub struct EguiRendererComponent {
    /// Deriving from [`Component`] requires that this is `Send + Sync`.
    pub renderer: Box<dyn EguiRenderer + Send + Sync>
}
