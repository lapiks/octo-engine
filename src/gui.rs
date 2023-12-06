use egui_wgpu::renderer::ScreenDescriptor;

use crate::{renderer_context::{Frame, RenderPassDesc, RendererContext, RenderPipelineHandle, PipelineDesc}, system::System};


pub struct Gui {
    ctx: egui::Context,
    gui_renderer: egui_wgpu::Renderer,
}

impl Gui {
    pub fn new(renderer: &RendererContext) -> Self {
        let gui_renderer = egui_wgpu::Renderer::new(
            renderer.get_device(), 
            wgpu::TextureFormat::Rgba8Unorm,
            None,
            1
        );

        Self {
            ctx: egui::Context::default(),
            gui_renderer,
        }
    }
}

impl System for Gui {
    fn init(&mut self, renderer: &mut crate::renderer_context::RendererContext) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn prepare_rendering(&mut self, renderer: &mut RendererContext) {

    }

    fn render(&mut self, frame: &mut Frame) {
        let mut name = "Arthur".to_owned();
        let mut age = 42;

        let raw_input = egui::RawInput::default();
        let full_output = self.ctx.run(raw_input, |ctx| {
            egui::CentralPanel::default().show(&self.ctx, |ui| {
                ui.heading("My egui Application");
                ui.horizontal(|ui| {
                    let name_label = ui.label("Your name: ");
                    ui.text_edit_singleline(&mut name)
                        .labelled_by(name_label.id);
                });
                ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    age += 1;
                }
                ui.label(format!("Hello '{name}', age {age}"));
            });
        });

        let clipped_primitives = self.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        // let pass = frame.begin_render_pass(
        //     &RenderPassDesc {
        //         bindings: &[],
        //         pipeline: ,
        //     }
        // );

        // self.gui_renderer.render(
        //     &mut pass.get_pass(),
        //     &clipped_primitives[..],
        //     &ScreenDescriptor {
        //         size_in_pixels: [frame.get_resolution().width, frame.get_resolution().height],
        //         pixels_per_point: full_output.pixels_per_point,
        //     }
        // );
    }

    fn resize(&mut self, renderer: &mut crate::renderer_context::RendererContext, width: u32, height: u32) {
        todo!()
    }

    fn on_key_down(&mut self, key: winit::event::VirtualKeyCode) {
        todo!()
    }

    fn on_key_up(&mut self, key: winit::event::VirtualKeyCode) {
        todo!()
    }

    fn on_mouse_button_down(&mut self, button: winit::event::MouseButton) {
        todo!()
    }

    fn on_mouse_button_up(&mut self, button: winit::event::MouseButton) {
        todo!()
    }

    fn on_mouse_move(&mut self, x_delta: f32, y_delta: f32) {
        todo!()
    }

    fn on_mouse_wheel(&mut self, delta: f32) {
        todo!()
    }
}