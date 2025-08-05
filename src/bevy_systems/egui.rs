use bevy_ecs::system::*;
use egui_wgpu::*;
use std::iter::*;
use wgpu::*;
use crate::bevy_components::egui::*;
use crate::bevy_resources::egui::*;
use crate::bevy_resources::wgpu::*;
use crate::bevy_resources::winit::*;

#[expect(clippy::needless_pass_by_value, reason = "bevy_ecs requires that Res system parameters be passed by value.")]
pub fn initialize_egui_system(
    wgpu_resource: Res<'_, WgpuResource>,
    winit_resource: Res<'_, WinitResource>,
    mut commands: Commands<'_, '_>
) {
    let egui_renderer_resource = EguiRendererResource::new(&EguiRendererResourceDescriptor {
        device: &wgpu_resource.device,
        msaa_samples: 1,
        output_color_format: wgpu_resource.surface_config.format,
        output_depth_format: None,
        window: &winit_resource.window
    });
    commands.insert_resource(egui_renderer_resource);
}

#[expect(clippy::needless_pass_by_value, reason = "bevy_ecs requires that Res system parameters be passed by value.")]
pub fn render_egui_system(
    mut egui_renderer_resource: ResMut<'_, EguiRendererResource>,
    wgpu_resource: Res<'_, WgpuResource>,
    wgpu_frame_resource: Res<'_, WgpuFrameResource>,
    winit_resource: Res<'_, WinitResource>,
    mut egui_state_resource: ResMut<'_, EguiStateResource>,
    egui_renderers: Query<'_, '_, &mut EguiRendererComponent>
) {
    let output_surface_texture_view = wgpu_frame_resource.output_surface_texture.texture.create_view(&TextureViewDescriptor {
        label: Some("egui-surface-texture-view"),
        ..Default::default()
    });
    let mut command_encoder = wgpu_resource.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("egui-command-encoder")
    });
    let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("egui-render-pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &output_surface_texture_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store
            }
        })],
        ..Default::default()
    }).forget_lifetime();
    for mut egui_renderer in egui_renderers {
        egui_renderer_resource.render_ui(&mut EguiRenderingDescriptor {
            window: &winit_resource.window,
            device: &wgpu_resource.device,
            queue: &wgpu_resource.command_queue,
            command_encoder: &mut command_encoder,
            render_pass: &mut render_pass,
            screen_descriptor: &ScreenDescriptor {
                size_in_pixels: [winit_resource.window.inner_size().width, winit_resource.window.inner_size().height],
                #[expect(clippy::cast_possible_truncation, reason = "pixels_per_point wants a f32.")]
                pixels_per_point: winit_resource.window.scale_factor() as f32
            },
            egui_renderer: &mut *egui_renderer.renderer,
            egui_state: &mut *egui_state_resource.egui_state
        });
    }
    drop(render_pass); // Remember to drop because of the forget_lifetime above; otherwise wgpu will panic.
    wgpu_resource.command_queue.submit(once(command_encoder.finish()));
}
