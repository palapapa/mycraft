use {
    crate::material::{Material, MaterialPipelineKey},
    bevy_ecs::resource::Resource,
    std::{collections::HashMap, sync::Arc},
    wgpu::{
        BindGroup, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration, SurfaceTexture
    },
    winit::window::Window
};

#[derive(Resource)]
pub struct WgpuResource {
    /// This has to be dropped before
    /// [`WinitResource::window`][crate::resources::winit::WinitResource::window]
    /// so that it is dropped before it in order to not cause a segfault. See
    /// <https://github.com/gfx-rs/wgpu/pull/1792>.
    ///
    /// The lifetime parameter of [`Surface`] is the lifetime of the
    /// [`Window`][winit::window::Window] used to create it, and since we
    /// wrapped the
    /// [`WinitResource::window`][crate::resources::winit::WinitResource::window]
    /// in an [`Arc`][std::sync::Arc], it can be `'static`. It has to be
    /// `'static` anyway, because a [`Resource`] must be `'static`.
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub device: Device,
    pub command_queue: Queue
}

/// These resources need to be recreated per frame, so it's easier to put them
/// in a separate [`Resource`].
#[derive(Resource)]
pub struct WgpuFrameResource {
    pub output_surface_texture: SurfaceTexture
}

#[derive(Resource)]
pub struct WinitResource {
    /// <https://www.reddit.com/r/rust/comments/1csjakb/comment/l45os9v>
    pub window: Arc<Window>
}

#[derive(Resource)]
pub struct RenderPipelineCacheResource<M: Material> {
    cache: HashMap<MaterialPipelineKey<M>, RenderPipeline>
}

#[derive(Resource)]
pub struct BindGroupCacheResource<M: Material> {
    cache: HashMap<MaterialPipelineKey<M>, BindGroup>
}
