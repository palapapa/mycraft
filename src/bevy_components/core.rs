use bevy_ecs::component::*;
use bevy_ecs::hierarchy::*;
use glam::*;
use crate::asset::*;
use crate::camera::*;
use crate::material::*;
use crate::mesh::*;

#[derive(Component, Default)]
#[require(GlobalTransformComponent)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

#[derive(Component, Default)]
#[component(on_insert = validate_parent_has_component::<GlobalTransformComponent>)]
pub struct GlobalTransformComponent {
    /// The global (world) transform of an [`bevy_ecs::entity::Entity`].
    global_transform: Affine3A
}

#[derive(Component)]
#[require(TransformComponent)]
pub struct CameraComponent {
    pub projection_mode: ProjectionMode
}

#[derive(Component)]
#[require(TransformComponent)]
pub struct MeshRendererComponent {
    pub material: AssetHandle<dyn Material + Send + Sync>,
    pub mesh: AssetHandle<Mesh>
}
