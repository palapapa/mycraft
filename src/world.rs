//! Contains functions to create the [`World`]s that the app uses.

use {
    crate::{
        components::egui::EguiRendererComponent,
        constants::ASSETS_PATH,
        egui_renderer::DefaultEguiRenderer,
        egui_state::DefaultEguiState,
        resources::{core::AssetCacheResource, egui::EguiStateResource}
    },
    assets_manager::AssetCache,
    bevy_ecs::world::World,
    log::{error, info},
    std::io,
    thiserror::Error
};

pub fn create_main_world() -> Result<World, WorldInitializationError> {
    let mut world = World::new();
    add_egui_entities_and_resources(&mut world);
    if let Err(err) = add_asset_cache_resources(&mut world) {
        return Err(err.into());
    }
    Ok(world)
}

fn add_egui_entities_and_resources(world: &mut World) {
    world.spawn(EguiRendererComponent {
        renderer: Box::new(DefaultEguiRenderer)
    });
    world.insert_resource(EguiStateResource {
        egui_state: Box::new(DefaultEguiState::new())
    });
}

fn add_asset_cache_resources(world: &mut World) -> Result<(), io::Error> {
    info!("Using assets path: {ASSETS_PATH}");
    let asset_cache = match AssetCache::new(ASSETS_PATH) {
        Ok(asset_cache) => asset_cache,
        Err(err) => {
            error!(
                "The World cannot be initialized because the \"{ASSETS_PATH}\" directory does not \
                 exist or is unreadable."
            );
            return Err(err);
        }
    };
    world.insert_resource(AssetCacheResource { asset_cache });
    Ok(())
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum WorldInitializationError {
    AssetsNotFound(#[from] io::Error)
}
