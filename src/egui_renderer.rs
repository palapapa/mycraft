use egui::*;
use egui_wgpu::*;
use egui_winit::*;
use crate::ui_renderer::*;
use crate::ui_state::*;
use wgpu::*;
use winit::window::{Window, Theme};
use winit::event::*;

/// Handles the lower level details of rendering the UI using [`egui`]. For the
/// actual UI renderers, see [`crate::ui_renderer`].
/// 
/// Inspired by <https://github.com/kaphula/winit-egui-wgpu-template>.
pub struct EguiRenderer {
    state: State,
    renderer: Renderer
}

impl EguiRenderer {
    /// * `egui_renderer_descriptor`: [`egui`] passes
    ///   [`EguiRendererDescriptor::output_depth_format`] to
    ///   [`Device::create_render_pipeline`] when it creates its own pipeline,
    ///   so it must match the format of the depth attachment you passed to
    ///   [`CommandEncoder::begin_render_pass`.]
    pub fn new(egui_renderer_descriptor: &EguiRendererDescriptor<'_>) -> Self {
        let &EguiRendererDescriptor { window, device, output_color_format, output_depth_format, msaa_samples } = egui_renderer_descriptor;
        let context = Context::default();
        // The meaning of the argument max_texture_side is documented at
        // egui::data::input::RawInput::max_texture_side.
        #[expect(clippy::cast_possible_truncation, reason = "State::new wants a f32.")]
        let state = State::new(context, ViewportId::ROOT, window, Some(window.scale_factor() as f32), Some(Theme::Dark), Some(device.limits().max_texture_dimension_2d as usize));
        let renderer = Renderer::new(device, output_color_format, output_depth_format, msaa_samples, true);
        Self {
            state,
            renderer,
        }
    }

    /// * `ui_rendering_descriptor`: [`Renderer::render`] requires that the
    ///   lifetime be `'static`, so you must call
    ///   [`RenderPass::forget_lifetime`] on
    ///   [`UiRenderingDescriptor::render_pass`] before passing.
    /// 
    ///   The `'static` in the type of [`UiRenderingDescriptor::ui_renderers`]
    ///   exists because the type of [`crate::App::ui_renderers`] is
    ///   `Vec<Box<dyn UiRenderer>>`, which is the same as `Vec<Box<dyn
    ///   UiRenderer + 'static>>` because of the lifetime elision rules.  Since
    ///   `&'a mut T` is invariant over `T` (see
    ///   [Rustonomicon](https://doc.rust-lang.org/nomicon/subtyping.html)), it
    ///   has to be `'static` when it's passed to here as well.
    pub fn render_ui(&mut self, ui_rendering_descriptor: &mut UiRenderingDescriptor<'_>) {
        // The immutable references don't need `ref mut` because immutable references are Copy.
        let &mut UiRenderingDescriptor { window, device, queue, ref mut command_encoder, ref mut render_pass, screen_descriptor, ref mut ui_renderers, ref mut ui_state } = ui_rendering_descriptor;
        self.state.egui_ctx().set_pixels_per_point(screen_descriptor.pixels_per_point);
        let raw_input = self.state.take_egui_input(window);
        let rendering_closure = |context: &Context| { // The explicit parameter type is required here; otherwise it causes the "implementation is not general enough" error
            for ui_renderer in &mut **ui_renderers { // The manual reborrow is required here
                ui_renderer.render_ui(context, *ui_state);
            }
        };
        let full_output = self.state.egui_ctx().run(raw_input, rendering_closure);
        self.state.handle_platform_output(window, full_output.platform_output);
        let primitives = self.state.egui_ctx().tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
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
        for id in full_output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
    }

    /// Should be called in
    /// [`winit::application::ApplicationHandler::window_event`].
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }
}

pub struct EguiRendererDescriptor<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub output_color_format: TextureFormat,
    pub output_depth_format: Option<TextureFormat>,
    pub msaa_samples: u32
}

pub struct UiRenderingDescriptor<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub command_encoder: &'a mut CommandEncoder,
    pub render_pass: &'a mut RenderPass<'static>,
    pub screen_descriptor: &'a ScreenDescriptor,
    pub ui_renderers: &'a mut [&'a mut (dyn UiRenderer + 'static)],
    pub ui_state: &'a mut dyn UiState
}
