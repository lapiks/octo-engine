use std::sync::Arc;

use egui::TopBottomPanel;
use egui_wgpu::ScreenDescriptor;
use winit::window::Window;

use crate::renderer_context::{Frame, RendererContext};


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

pub fn run_ui(ctx: &egui::Context) {
    // TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
    //     egui::menu::bar(ui, |ui| {
    //         ui.menu_button("View", |ui| {
    //             // allow certain tabs to be toggled
    //             for tab in &["File Browser", "Asset Manager"] {
    //                 if ui
    //                     .selectable_label(self.context.open_tabs.contains(*tab), *tab)
    //                     .clicked()
    //                 {
    //                     if let Some(index) = self.tree.find_tab(&tab.to_string()) {
    //                         self.tree.remove_tab(index);
    //                         self.context.open_tabs.remove(*tab);
    //                     } else {
    //                         self.tree[SurfaceIndex::main()]
    //                             .push_to_focused_leaf(tab.to_string());
    //                     }

    //                     ui.close_menu();
    //                 }
    //             }
    //         });
    //     })
    // });
    // CentralPanel::default()
    //     // When displaying a DockArea in another UI, it looks better
    //     // to set inner margins to 0.
    //     .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
    //     .show(ctx, |ui| {
    //         let style = self
    //             .context
    //             .style
    //             .get_or_insert(Style::from_egui(ui.style()))
    //             .clone();

    //         DockArea::new(&mut self.tree)
    //             .style(style)
    //             .show_close_buttons(self.context.show_close_buttons)
    //             .show_add_buttons(self.context.show_add_buttons)
    //             .draggable_tabs(self.context.draggable_tabs)
    //             .show_tab_name_on_hover(self.context.show_tab_name_on_hover)
    //             .allowed_splits(self.context.allowed_splits)
    //             .show_window_close_buttons(self.context.show_window_close)
    //             .show_window_collapse_buttons(self.context.show_window_collapse)
    //             .show_inside(ui, &mut self.context);
    //     });
}