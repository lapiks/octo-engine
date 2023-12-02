use crate::renderer_context::RendererContext;

pub trait System {
    fn init(&mut self, renderer: &mut RendererContext);
    fn update(&mut self);
    fn render(&mut self, renderer: &mut RendererContext);
    fn resize(&mut self, renderer: &mut RendererContext, width: u32, height: u32);
    fn on_key_down(&mut self, key: winit::event::VirtualKeyCode);
    fn on_key_up(&mut self, key: winit::event::VirtualKeyCode);
    fn on_mouse_button_down(&mut self, button: winit::event::MouseButton);
    fn on_mouse_button_up(&mut self, button: winit::event::MouseButton);
    fn on_mouse_move(&mut self, xDelta: f32, yDelta: f32);
    fn on_mouse_wheel(&mut self, delta: f32);
}



