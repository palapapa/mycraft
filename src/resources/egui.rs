use bevy_ecs::resource::*;
use egui::*;
use egui_wgpu::*;
use egui_winit::*;
use std::sync::*;
use wgpu::*;
use winit::event::*;
use winit::window::{Window, Theme};
use crate::egui_state::*;
use crate::egui_renderer::*;
use crate::extensions::*;

#[derive(Resource)]
pub struct EguiStateResource {
    /// Deriving from [`Resource`] requires that this is `Send + Sync`.
    pub egui_state: Box<dyn EguiState + Send + Sync>
}

/// Handles the lower level details of rendering the UI using [`egui`]. For the
/// actual UI renderers, see [`crate::egui_renderer`].
/// 
/// Based on <https://github.com/kaphula/winit-egui-wgpu-template>.
/// 
/// This has to be dropped before
/// [`crate::resources::winit::WinitResource::window`] and
/// [`crate::resources::wgpu::WgpuResource::device`] because if this is dropped
/// after them, the app will segfault. See
/// <https://github.com/emilk/egui/issues/7369>.
#[derive(Resource)]
pub struct EguiRendererResource {
    /// Wrapped in a [`Mutex`] because [`State`] is not [`Sync`], but it needs
    /// to be so [`EguiRendererResource`] can be made a [`Resource`].
    state: Mutex<State>,
    renderer: Renderer
}

impl EguiRendererResource {
    /// * `egui_renderer_descriptor`: [`egui`] passes
    ///   [`EguiRendererResourceDescriptor::output_depth_format`] to
    ///   [`Device::create_render_pipeline`] when it creates its own pipeline,
    ///   so it must match the format of the depth attachment you passed to
    ///   [`CommandEncoder::begin_render_pass`].
    pub fn new(egui_renderer_descriptor: &EguiRendererResourceDescriptor<'_>) -> Self {
        let &EguiRendererResourceDescriptor { window, device, output_color_format, output_depth_format, msaa_samples } = egui_renderer_descriptor;
        let context = Context::default();
        // The meaning of the argument max_texture_side is documented at
        // egui::data::input::RawInput::max_texture_side.
        #[expect(clippy::cast_possible_truncation, reason = "State::new wants a f32.")]
        let state = State::new(context, ViewportId::ROOT, window, Some(window.scale_factor() as f32), Some(Theme::Dark), Some(device.limits().max_texture_dimension_2d as usize));
        let renderer = Renderer::new(device, output_color_format, output_depth_format, msaa_samples, true);
        Self {
            state: Mutex::new(state),
            renderer,
        }
    }

    /// * `egui_rendering_descriptor`: [`Renderer::render`] requires that the
    ///   lifetime be `'static`, so you must call
    ///   [`RenderPass::forget_lifetime`] on
    ///   [`EguiRenderingDescriptor::render_pass`] before passing.
    pub fn render_ui(&mut self, egui_rendering_descriptor: &mut EguiRenderingDescriptor<'_>) {
        // The immutable references don't need `ref mut` because immutable references are Copy.
        let &mut EguiRenderingDescriptor { window, device, queue, ref mut command_encoder, ref mut render_pass, screen_descriptor, ref mut egui_renderer, ref mut egui_state } = egui_rendering_descriptor;
        let mut state = self.state.lock_and_unwrap();
        state.egui_ctx().set_pixels_per_point(screen_descriptor.pixels_per_point);
        let raw_input = state.take_egui_input(window);
        let rendering_closure = |context: &Context| { // The explicit parameter type is required here; otherwise it causes the "implementation is not general enough" error
            egui_renderer.render_ui(context, *egui_state);
        };
        let full_output = state.egui_ctx().run(raw_input, rendering_closure);
        state.handle_platform_output(window, full_output.platform_output);
        let primitives = state.egui_ctx().tessellate(full_output.shapes, state.egui_ctx().pixels_per_point());
        drop(state);
        for (id, image_delta) in full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, id, &image_delta);
        }
        self.renderer.update_buffers(device, queue, command_encoder, &primitives, screen_descriptor);
        self.renderer.render(render_pass, &primitives, screen_descriptor);
        // Free textures marked for destruction **after** queue submit since
        // they might still be used in the current frame. Calling
        // `wgpu::Texture::destroy` on a texture that is still in use would
        // invalidate the command buffer(s) it is used in. However, once we
        // called `wgpu::Queue::submit`, it is up for wgpu to determine how long
        // the underlying gpu resource has to live.
        //
        // (Excerpt from
        // https://github.com/emilk/egui/blob/main/crates/egui-wgpu/src/winit.rs)
        for id in full_output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
    }

    /// Should be called in
    /// [`winit::application::ApplicationHandler::window_event`].
    pub fn handle_event(&self, window: &Window, event: &WindowEvent) -> EventResponse {
        let mut state = self.state.lock_and_unwrap();
        state.on_window_event(window, event)
    }
}

pub struct EguiRendererResourceDescriptor<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub output_color_format: TextureFormat,
    pub output_depth_format: Option<TextureFormat>,
    pub msaa_samples: u32
}

pub struct EguiRenderingDescriptor<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub command_encoder: &'a mut CommandEncoder,
    pub render_pass: &'a mut RenderPass<'static>,
    pub screen_descriptor: &'a ScreenDescriptor,
    pub egui_renderer: &'a mut dyn EguiRenderer,
    pub egui_state: &'a mut dyn EguiState
}
