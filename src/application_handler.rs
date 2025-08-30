use bevy_ecs::schedule::*;
use bevy_ecs::system::*;
use bevy_ecs::world::*;
use egui_wgpu::*;
use std::fmt::Debug;
use std::iter::*;
use std::sync::*;
use thiserror::*;
use futures::executor::*;
use log::*;
use tokio::task::block_in_place;
use winit::application::*;
use winit::dpi::*;
use winit::error::*;
use winit::event_loop::*;
use winit::window::*;
use winit::event::WindowEvent::{self, *};
use wgpu::*;
use crate::bevy_resources::egui::*;
use crate::bevy_resources::wgpu::*;
use crate::bevy_resources::winit::*;
use crate::bevy_schedules::*;
use crate::bevy_world::*;

/// An implementation of [`ApplicationHandler`] that manages the states of the app and the GPU.
pub struct App {
    world: World,
    startup_schedule: Schedule,
    update_schedule: Schedule,
    is_initialized: bool
}

impl App {
    pub fn new() -> Result<Self, WorldInitializationError> {
        Ok(Self {
            world: create_main_world()?,
            startup_schedule: StartupSchedule::create_schedule(),
            update_schedule: UpdateSchedule::create_schedule(),
            is_initialized: false
        })
    }
    
    /// Initializes the fields inside this `struct` and the ECS. Does nothing if
    /// called multiple times after succeeding. This should be called in
    /// [`ApplicationHandler::resumed`].
    /// 
    /// * `event_loop`: The [`ActiveEventLoop`] provided by
    ///   [`ApplicationHandler::resumed`] used to create a [`Window`].
    #[expect(clippy::future_not_send, reason = "This function is currently only called using block_on.")]
    async fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), AppInitializationError> {
        if self.is_initialized {
            return Ok(());
        }
        let window = Self::create_window(event_loop)?;
        let wgpu_instance = Self::create_instance();
        let surface = Self::create_surface(&wgpu_instance, Arc::clone(&window))?;
        let adapter = Self::create_adapter(&wgpu_instance, &surface).await?;
        let (device, command_queue) = Self::create_device_and_queue(&adapter).await?;
        let surface_config = Self::create_surface_config(&surface, &adapter, &window)?;
        surface.configure(&device, &surface_config);
        info!("Surface configured.");
        self.world.insert_resource(WgpuResource { command_queue, surface, surface_config, device });
        self.world.insert_resource(WinitResource { window });
        self.startup_schedule.run(&mut self.world);
        self.is_initialized = true;
        Ok(())
    }

    fn create_window(event_loop: &ActiveEventLoop) -> Result<Arc<Window>, AppInitializationError> {
        let window_attributes = Window::default_attributes().with_title("Mycraft");
        match event_loop.create_window(window_attributes) {
            Ok(window) => {
                info!("The window with ID {} has been created.", u64::from(window.id()));
                Ok(Arc::new(window))
            },
            Err(err) => {
                error!("Could not create the window. {err:#?}");
                Err(AppInitializationError::Os(err))
            }
        }
    }

    fn create_instance() -> Instance {
        Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::from_env_or_default(),
            ..Default::default()
        })
    }

    fn create_surface<'window>(instance: &Instance, window: impl Into<SurfaceTarget<'window>>) -> Result<Surface<'window>, AppInitializationError> {
        match instance.create_surface(window) {
            Ok(surface) => {
                info!("The surface has been created. {surface:#?}");
                Ok(surface)
            },
            Err(err) => {
                error!("Could not create the surface. {err:#?}");
                Err(AppInitializationError::CreateSurface(err))
            }
        }
    }

    async fn create_adapter(instance: &Instance, surface: &Surface<'_>) -> Result<Adapter, AppInitializationError> {
        match instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false
        }).await {
            Ok(adapter) => {
                info!("The adapter has been created. {adapter:#?}");
                info!("Adapter limits: {:#?}", adapter.limits());
                Ok(adapter)
            }
            Err(err) => {
                error!("The adapter could not be created. {err:#?}");
                Err(AppInitializationError::RequestAdapter(err))
            }
        }
    }

    async fn create_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue), AppInitializationError> {
        match adapter.request_device(&DeviceDescriptor {
            label: Some("default-device"),
            required_limits: adapter.limits(),
            ..Default::default()
        }).await {
            Ok(val) => {
                info!("The device and command queue has been created. {:#?}, {:#?}", val.0, val.1);
                Ok(val)
            },
            Err(err) => {
                error!("The device and command queue could not be created. {err:#?}");
                Err(AppInitializationError::RequestDevice(err))
            }
        }
    }

    fn create_surface_config(surface: &Surface<'_>, adapter: &Adapter, window: &Window) -> Result<SurfaceConfiguration, AppInitializationError> {
        let surface_capabilities = surface.get_capabilities(adapter);
        if surface_capabilities.formats.is_empty() {
            error!("The surface format could not be determined because the surface is incompatible with the adapter.");
            return Err(AppInitializationError::CreateSurfaceTextureFormat);
        }
        #[expect(clippy::indexing_slicing, reason = "The Vec has already been checked to not be empty.")]
        let surface_format = surface_capabilities.formats
            .iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        info!("Supported surface formats: {:#?}", surface_capabilities.formats);
        info!("The surface format {surface_format:#?} has been chosen.");
        let present_mode = PresentMode::Fifo;
        info!("Supported present modes: {:#?}", surface_capabilities.present_modes);
        info!("The present mode {present_mode:#?} has been chosen.");
        let alpha_mode = CompositeAlphaMode::Auto;
        info!("Supported alpha modes: {:#?}", surface_capabilities.alpha_modes);
        info!("The alpha mode {alpha_mode:#?} has been chosen.");
        let surface_usages = TextureUsages::RENDER_ATTACHMENT;
        info!("Supported surface usages: {:#?}", surface_capabilities.usages);
        info!("The surface usages {surface_usages:#?} have been chosen.");
        Ok(SurfaceConfiguration {
            alpha_mode,
            desired_maximum_frame_latency: 2,
            format: surface_format,
            height: window.inner_size().height,
            width: window.inner_size().width,
            present_mode,
            usage: surface_usages,
            view_formats: vec![]
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if !self.is_initialized {
            return;
        }
        let wgpu_resource: &mut WgpuResource = &mut self.world.resource_mut();
        wgpu_resource.surface_config.width = new_size.width;
        wgpu_resource.surface_config.height = new_size.height;
        wgpu_resource.surface.configure(&wgpu_resource.device, &wgpu_resource.surface_config);
        info!("Resized the window to {new_size:#?}");
    }

    fn render(&mut self) -> Result<(), RenderError> {
        if !self.is_initialized {
            return Ok(());
        }
        let wgpu_resource: &WgpuResource = self.world.resource();
        let output_surface_texture = match wgpu_resource.surface.get_current_texture() {
            Ok(output_surface_texture) => output_surface_texture,
            Err(err) => {
                error!("`get_current_texture` failed when rendering. {err:#?}");
                return Err(RenderError::Surface(err))
            }
        };
        let output_surface_texture_view = output_surface_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("main-surface-texture-view"),
            ..Default::default()
        });
        let mut command_encoder = wgpu_resource.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("main-command-encoder")
        });
        let render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("main-render-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &output_surface_texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::default()),
                    store: StoreOp::Store
                }
            })],
            ..Default::default()
        });
        drop(render_pass); // Drops render_pass so command_encoder can be moved below.
        // Acquires WgpuResource again. This somehow solves a double mutable
        // reference error.
        let wgpu_resource_2: &WgpuResource = self.world.resource();
        wgpu_resource_2.command_queue.submit(once(command_encoder.finish()));
        self.world.insert_resource(WgpuFrameResource { output_surface_texture });
        self.update_schedule.run(&mut self.world);
        let winit_resource: &WinitResource = self.world.resource();
        winit_resource.window.pre_present_notify();
        let returned_output_surface_texture = if let Some(wgpu_frame_resource) = self.world.remove_resource::<WgpuFrameResource>() {
                wgpu_frame_resource.output_surface_texture
            }
            else {
                error!("Could not retrieve the SurfaceTexture to present after MainSchedule finished.");
                return Err(RenderError::CouldNotPresent)
            };
        returned_output_surface_texture.present();
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.is_initialized {
            let winit_resource: &WinitResource = self.world.resource();
            info!("The window with ID {} is resuming.", u64::from(winit_resource.window.id()));
            return;
        }
        // If `resumed` is called for the first time.
        match block_in_place(|| block_on(self.initialize(event_loop))) {
            Ok(()) => {
                info!("The window has been initialized.");
            },
            Err(err) => {
                error!("Failed to initialize the window. {err:#?}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if !self.is_initialized {
            return;
        }
        // We need to use a SystemSet here to avoid having a mutable and
        // immutable borrow of self.world at the same time.
        let mut system_state: SystemState<
            (Res<'_, EguiRendererResource>,
            Res<'_, WinitResource>)> = SystemState::new(&mut self.world);
        let (egui_renderer_resource, winit_resource) = system_state.get(&self.world);
        let event_response = egui_renderer_resource.handle_event(&winit_resource.window, &event);
        if event_response.consumed {
            return;
        }
        match event {
            Resized(new_size) => {
                self.resize(new_size);
            }
            RedrawRequested => {
                winit_resource.window.request_redraw(); // Can't be below self.render because that would create two mutable references.
                match self.render() {
                    Ok(()) => (),
                    Err(err) => {
                        error!("Error encountered while rendering. {err:#?}");
                    }
                }
            }
            CloseRequested => {
                info!("The window with ID {} is exiting.", u64::from(window_id));
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        // Removes these three resources in this very particular order to
        // prevent a segfault.
        if let Some(egui_renderer_resource) = self.world.remove_resource::<EguiRendererResource>() {
            drop(egui_renderer_resource);
        }
        if let Some(wgpu_resource) = self.world.remove_resource::<WgpuResource>() {
            drop(wgpu_resource);
        }
        if let Some(winit_resource) = self.world.remove_resource::<WinitResource>() {
            drop(winit_resource);
        }
    }
}

#[derive(Error, Debug)]
enum AppInitializationError {
    #[error(transparent)]
    Os(#[from] OsError),

    #[error(transparent)]
    CreateSurface(#[from] CreateSurfaceError),

    #[error(transparent)]
    RequestDevice(#[from] RequestDeviceError),

    #[error("The surface format could not be determined because the surface is incompatible with the adapter.")]
    CreateSurfaceTextureFormat,

    #[error(transparent)]
    RequestAdapter(#[from] RequestAdapterError)
}

#[derive(Error, Debug)]
enum RenderError {
    #[error(transparent)]
    Surface(#[from] SurfaceError),

    #[error("Could not retrieve the SurfaceTexture to present after MainSchedule finished.")]
    CouldNotPresent
}
