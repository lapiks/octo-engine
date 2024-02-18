use winit::{event::MouseButton, keyboard::KeyCode};

use crate::renderer_context::{RendererContext, Frame};

pub trait System {
    fn init(&mut self, renderer: &mut RendererContext);
    fn update(&mut self);
    fn prepare_rendering(&mut self, renderer: &mut RendererContext);
    fn render(&mut self, frame: &mut Frame);
    fn resize(&mut self, renderer: &mut RendererContext, width: u32, height: u32);
    fn on_key_down(&mut self, key: KeyCode);
    fn on_key_up(&mut self, key: KeyCode);
    fn on_mouse_button_down(&mut self, button: MouseButton);
    fn on_mouse_button_up(&mut self, button: MouseButton);
    fn on_mouse_move(&mut self, x_delta: f32, y_delta: f32);
    fn on_mouse_wheel(&mut self, delta: f32);
}



