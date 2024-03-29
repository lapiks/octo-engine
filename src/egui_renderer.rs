use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use winit::window::Window;

use crate::renderer_context::{Frame, RendererContext, TextureHandle};


pub struct EguiRenderer {
    egui_context: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    egui_state: egui_winit::State,
}

impl EguiRenderer {
    pub fn new(renderer: &RendererContext, window: Arc<Window>) -> Self {
        let egui_renderer = egui_wgpu::Renderer::new(
            renderer.device(), 
            wgpu::TextureFormat::Bgra8UnormSrgb,
            None,
            1
        );

        let egui_context = egui::Context::default();
        let id = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visuals = egui::Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: egui::epaint::Shadow::NONE,
            // menu_rounding: todo!(),
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = egui_winit::State::new(egui_context.clone(), id, &window, None, None);

        Self {
            egui_context,
            egui_renderer,
            egui_state,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &winit::event::WindowEvent) {
        let _ = self.egui_state.on_window_event(window, event);
    }

    pub fn render(
        &mut self,
        renderer: &RendererContext,
        frame: &mut Frame,
        window: &Window,
        run_ui: impl FnOnce(&egui::Context)
    ) {
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_context.run(raw_input, |ctx| {
            run_ui(ctx);
        });

        let clipped_primitives = self.egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(renderer.device(), renderer.queue(), *id, &image_delta);
        }

        let size = window.inner_size();
        let screen_desc = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        self.egui_renderer.update_buffers(
            renderer.device(),
            renderer.queue(),
            frame.encoder_mut(),
            &clipped_primitives,
            &screen_desc,
        );

        let mut rpass = frame.new_render_pass(wgpu::LoadOp::Load);
        self.egui_renderer.render(
            &mut rpass,
            &clipped_primitives[..],
            &screen_desc,
        );

        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(x)
        }
    }

    pub fn register_native_texture(&mut self, renderer: &RendererContext, game_texture: TextureHandle) -> Option<egui::TextureId> {
        renderer.get_texture(game_texture).and_then(|texture| {
            Some(self.egui_renderer.register_native_texture(
                renderer.device(), 
                &texture.view, 
                wgpu::FilterMode::Nearest
            ))
        })
    }
}
