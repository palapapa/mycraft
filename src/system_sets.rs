use bevy_ecs::schedule::*;

/// A collection of [`SystemSet`]s related to rendering.
#[derive(Clone, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum RenderingSystemSets {
    
    /// A [`SystemSet`] for rendering the [`egui`].
    Egui
}