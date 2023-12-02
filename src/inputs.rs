use std::collections::HashMap;

use glam::Vec2;
use winit::event::{VirtualKeyCode, MouseButton};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Default)]
pub struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
}

pub struct Inputs {
    mouse: HashMap<MouseButton, bool>,
    keys: HashMap<VirtualKeyCode, bool>,
    modifiers: Modifiers,
    mouse_delta: Vec2,
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            mouse: HashMap::default(),
            keys: HashMap::default(),
            modifiers: Modifiers::default(),
            mouse_delta: Vec2::ZERO,
        }
    } 

    pub fn reset(&mut self) {
        self.mouse_delta = Vec2::ZERO;
    }

    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_delta += Vec2::new(x, y);
    }

    pub fn on_mouse_wheel(&mut self, delta: f32) {

    }

    pub fn on_mouse_button_down(&mut self, button: MouseButton) {
        self.mouse.insert(button, true);
    }

    pub fn on_mouse_button_up(&mut self, button: MouseButton) {
        self.mouse.insert(button, false);
    }

    pub fn on_key_down(&mut self, keycode: VirtualKeyCode) {
        self.keys.insert(keycode, true);
    }

    pub fn on_key_up(&mut self, keycode: VirtualKeyCode) {
        self.keys.insert(keycode, false);
    }

    pub fn get_key_down(&self, keycode: VirtualKeyCode) -> bool {
        *self.keys.get(&keycode).unwrap_or(&false)
    }

    pub fn get_modifiers(&self) -> &Modifiers {
        &self.modifiers
    }

    pub fn get_button_down(&self, button: MouseButton) -> bool {
        *self.mouse.get(&button).unwrap_or(&false)
    }

    pub fn get_mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }
}