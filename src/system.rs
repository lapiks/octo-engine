use crate::renderer_context::RendererContext;

pub trait System {
    fn init(&mut self, renderer: &mut RendererContext);
    fn update(&mut self);
    fn render(&mut self, renderer: &mut RendererContext);
    fn resize(&mut self, renderer: &mut RendererContext, width: u32, height: u32);
    fn on_key_down(&mut self, key: winit::event::VirtualKeyCode);
}



