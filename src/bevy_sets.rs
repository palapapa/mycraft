use bevy_ecs::schedule::*;

/// A [`SystemSet`] used to render [`egui`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EguiSystemSet;
