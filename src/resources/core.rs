use assets_manager::*;
use bevy_ecs::resource::*;

/// The [`AssetCache`] contained only stores assets that are loaded from files
/// on the disk. Dynamically-generated assets are represented using
/// [`AssetHandle::Procedural`][crate::asset::AssetHandle::Procedural].
#[derive(Resource)]
pub struct AssetCacheResource {
    pub asset_cache: AssetCache
}
