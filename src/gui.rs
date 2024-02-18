use egui_wgpu::renderer::ScreenDescriptor;
use winit::window::Window;

use crate::renderer_context::{Frame, RendererContext};


pub struct EguiRenderer {
    egui_context: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    egui_state: egui_winit::State,
}

impl EguiRenderer {
    pub fn new(renderer: &RendererContext, window: &Window) -> Self {
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

        let screen_desc = ScreenDescriptor {
            size_in_pixels: [800, 600],
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
}

pub fn run_ui(ui: &egui::Context) {
    egui::Window::new("Editor")
        // .vscroll(true)
        .default_open(true)
        .max_width(1000.0)
        .max_height(800.0)
        .default_width(800.0)
        .resizable(true)
        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
        .show(&ui, |ui| {
            if ui.add(egui::Button::new("Click me")).clicked() {
                println!("PRESSED")
            }
        });
}