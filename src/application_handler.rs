use std::fmt::Debug;
use std::iter::once;
use std::sync::Arc;
use futures::executor::block_on;
use log::{error, info};
use tokio::task::block_in_place;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::error::OsError;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent::{self, *};
use wgpu::{Backends, Color, CommandEncoderDescriptor, CompositeAlphaMode, CreateSurfaceError, Device, DeviceDescriptor, Instance, InstanceDescriptor, InstanceFlags, LoadOp, Operations, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterError, RequestAdapterOptions, RequestDeviceError, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor};
use wgpu::PowerPreference::HighPerformance;

/// An implementation of [`ApplicationHandler`] that manages the states of the app and the GPU.
#[derive(Default)]
pub struct App<'a> {
    /// Because if these fields were placed in [`App`] directly, they would all have
    /// to be [`Option`]s, and all either be [`Some`] of [`None`] at the same time.
    /// So [`Option<AppInternalFields<'a>>`] is used in it instead.
    internal_fields: Option<AppInternalFields<'a>>
}

impl App<'_> {
    /// Initializes the fields inside [`Self::internal_fields`]. Does nothing if
    /// called multiple times after succeeding.
    /// 
    /// * `event_loop`: The [`ActiveEventLoop`] provided by
    ///   [`ApplicationHandler::resumed`] used to create a [`Window`] to store
    ///   inside [`Self::internal_fields`].
    #[expect(clippy::future_not_send, reason = "This function is currently only called using block_on.")]
    async fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), AppInitializationError> {
        if self.internal_fields.is_some() {
            return Ok(());
        }
        let window_attributes = Window::default_attributes().with_title("Mycraft");
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => {
                info!("The window with ID {} has been created.", u64::from(window.id()));
                Arc::new(window)
            },
            Err(err) => {
                error!("Could not create the window. {err:#?}");
                return Err(AppInitializationError::Os(err));
            }
        };
        let wgpu_instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::from_env_or_default(),
            ..Default::default()
        });
        let surface = match wgpu_instance.create_surface(Arc::clone(&window)) {
            Ok(surface) => {
                info!("The surface has been created. {surface:#?}");
                surface
            },
            Err(err) => {
                error!("Could not create the surface. {err:#?}");
                return Err(AppInitializationError::CreateSurface(err));
            }
        };
        let adapter = match wgpu_instance.request_adapter(&RequestAdapterOptions {
            power_preference: HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await {
            Ok(adapter) => {
                info!("The adapter has been created. {adapter:#?}");
                adapter
            }
            Err(err) => {
                error!("The adapter could not be created. {err:#?}");
                return Err(AppInitializationError::RequestAdapter(err));
            }
        };
        let (device, command_queue) = match adapter.request_device(&DeviceDescriptor {
            label: Some("default-device"),
            ..Default::default()
        }).await {
            Ok(val) => {
                info!("The device and command queue has been created. {:#?}, {:#?}", val.0, val.1);
                val
            },
            Err(err) => {
                error!("The device and command queue could not be created. {err:#?}");
                return Err(AppInitializationError::RequestDevice(err));
            }
        };
        let surface_capabilities = surface.get_capabilities(&adapter);
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
        let surface_config = SurfaceConfiguration {
            alpha_mode,
            desired_maximum_frame_latency: 2,
            format: surface_format,
            height: window.inner_size().height,
            width: window.inner_size().width,
            present_mode,
            usage: surface_usages,
            view_formats: vec![]
        };
        self.internal_fields = Some(AppInternalFields {
            command_queue,
            device,
            surface,
            surface_config,
            window
        });
        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let Some(internal_fields) = self.internal_fields.as_mut() else { return; };
        internal_fields.surface_config.width = new_size.width;
        internal_fields.surface_config.height = new_size.height;
        internal_fields.surface.configure(&internal_fields.device, &internal_fields.surface_config);
    }

    fn render(&self) -> Result<(), SurfaceError> {
        let Some(internal_fields) = self.internal_fields.as_ref() else { return Ok(()); };
        let output_surface_texture = match internal_fields.surface.get_current_texture() {
            Ok(output_surface_texture) => output_surface_texture,
            Err(err) => {
                error!("`get_current_texture` failed when rendering. {err:#?}");
                return Err(err)
            }
        };
        let output_surface_texture_view = output_surface_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("main-surface-texture-view"),
            ..Default::default()
        });
        let mut command_encoder = internal_fields.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("main-command-encoder")
        });
        command_encoder.begin_render_pass(&RenderPassDescriptor {
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
        internal_fields.command_queue.submit(once(command_encoder.finish()));
        output_surface_texture.present();
        Ok(())
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(internal_fields) = self.internal_fields.as_ref() {
            info!("The window with ID {} is resuming.", u64::from(internal_fields.window.id()));
            return;
        }
        // If `resumed` is called for the first time.
        match block_in_place(|| block_on(self.initialize(event_loop))) {
            Ok(()) => {
                info!("The window has been initialized.");
            },
            Err(err) => {
                error!("Failed to initialize the window. {err:#?}");
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            Resized(new_size) => {
                self.resize(new_size);
            },
            RedrawRequested => {
                _ = self.render();
            }
            CloseRequested => {
                info!("The window with ID {} is exiting.", u64::from(window_id));
                event_loop.exit();
            }
            _ => {}
        }
    }
}

/// Because if these fields were placed in [`App`] directly, they would all have
/// to be [`Option`]s, and all either be [`Some`] of [`None`] at the same time.
/// So [`Option<AppInternalFields<'a>>`] is used in it instead.
struct AppInternalFields<'a> {
    /// This has to be declared before [`Self::window`] so that it is dropped
    /// before it in order to not cause a segfault. See
    /// <https://github.com/gfx-rs/wgpu/pull/1792>.
    surface: Surface<'a>,
    /// <https://www.reddit.com/r/rust/comments/1csjakb/comment/l45os9v>
    /// 
    /// This also cannot be a owned [`Window`], because [`Self::surface`] holds
    /// a reference to this, but is created in the same scope as this. If this
    /// were a owned [`Window`] you would get a "borrowed data escapes outside
    /// of ..." error.
    window: Arc<Window>,
    surface_config: SurfaceConfiguration,
    device: Device,
    command_queue: Queue
}

#[derive(Debug)]
#[expect(dead_code, reason = "The tuple variants are currectly unused.")]
enum AppInitializationError {
    Os(OsError),
    CreateSurface(CreateSurfaceError),
    RequestDevice(RequestDeviceError),
    CreateSurfaceTextureFormat,
    RequestAdapter(RequestAdapterError)
}
