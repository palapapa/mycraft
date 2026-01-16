use bevy_ecs::component::*;
use bevy_ecs::hierarchy::*;
use glam::*;
use std::borrow::*;
use std::ops::*;
use crate::asset::*;
use crate::camera::*;
use crate::material::*;
use crate::mesh::*;

/// Represents an [`bevy_ecs::entity::Entity`]s transformation in its local
/// space. This [`Component`] must be accompanied by
/// [`GlobalTransformComponent`] and [`TransformTreeChangedComponent`], which
/// are added automatically whenever this [`Component`] is added. It is an error
/// to later remove any one of the three. Any entities that have a physical
/// position in the world should have this. If an entity has this, but some of
/// its parents all the way to the root doesn't have this, it will not be
/// rendered correctly.
#[derive(Component)]
#[require(GlobalTransformComponent, TransformTreeChangedComponent)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE
        }
    }
}

impl From<TransformComponent> for Affine3A {
    fn from(value: TransformComponent) -> Self {
        Self::from(&value)
    }
}

impl From<&TransformComponent> for Affine3A {
    fn from(value: &TransformComponent) -> Self {
        Self::from_scale_rotation_translation(value.scale, value.rotation, value.position)
    }
}

/// Represents an [`bevy_ecs::entity::Entity`]s transformation in the world
/// space. The value of this [`Component`] is managed by
/// [`crate::systems::transform`] automatically. Other places should not
/// directly modify its value.
#[derive(Component, Default, PartialEq)]
#[component(on_insert = validate_parent_has_component::<GlobalTransformComponent>)]
pub struct GlobalTransformComponent {
    /// The global (world) transform of an [`bevy_ecs::entity::Entity`].
    global_transform: Affine3A
}

impl GlobalTransformComponent {
    /// Transforms `transform` with the transformation stored in
    /// [`GlobalTransformComponent`].
    pub fn mul_transform(&self, transform: &TransformComponent) -> Self {
        self * Self::from(transform)
    }
}

impl<T: Borrow<Self>> Mul<T> for GlobalTransformComponent {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        &self * rhs.borrow()
    }
}

impl<T: Borrow<GlobalTransformComponent>> Mul<T> for &GlobalTransformComponent {
    type Output = GlobalTransformComponent;

    fn mul(self, rhs: T) -> Self::Output {
        GlobalTransformComponent { global_transform: self.global_transform * rhs.borrow().global_transform }
    }
}

impl<T: Borrow<TransformComponent>> From<T> for GlobalTransformComponent {
    fn from(value: T) -> Self {
        Self { global_transform: value.borrow().into() }
    }
}

/// A marker [`Component`] that acts as a dirty bit and uses change detection to
/// mark whether an [`bevy_ecs::entity::Entity`] or any of its descendants'
/// [`GlobalTransformComponent`] need to be recalculated. See
/// [`crate::systems::transform`] for more detail.
#[derive(Component, Default)]
pub struct TransformTreeChangedComponent;

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
