use egui::*;
use egui_wgpu::*;
use egui_winit::*;
use wgpu::*;
use winit::window::{Window, Theme};
use winit::event::*;

/// Handles everything related to drawing the UI using [`egui`]. Inspired by
/// <https://github.com/kaphula/winit-egui-wgpu-template>.
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

    /// * `ui_drawing_descriptor`: [`Renderer::render`] requires that the
    ///   lifetime be `'static`, so you must call
    ///   [`RenderPass::forget_lifetime`] on
    ///   [`UiDrawingDescriptor::render_pass`] before passing.
    pub fn draw_ui(&mut self, ui_drawing_descriptor: &mut UiDrawingDescriptor<'_>, ui_code: impl FnMut(&Context)) {
        let &mut UiDrawingDescriptor { window, device, queue, ref mut command_encoder, ref mut render_pass, screen_descriptor } = ui_drawing_descriptor;
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.state.egui_ctx().run(raw_input, ui_code);
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

pub struct UiDrawingDescriptor<'a> {
    pub window: &'a Window,
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub command_encoder: &'a mut CommandEncoder,
    pub render_pass: &'a mut RenderPass<'static>,
    pub screen_descriptor: &'a ScreenDescriptor
}
