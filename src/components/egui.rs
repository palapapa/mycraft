use {crate::egui_renderer::EguiRenderer, bevy_ecs::component::Component};

#[derive(Component)]
pub struct EguiRendererComponent {
    /// Deriving from [`Component`] requires that this is `Send + Sync`.
    pub renderer: Box<dyn EguiRenderer + Send + Sync>
}
