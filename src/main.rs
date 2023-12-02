mod renderer_context;
mod globals;
mod camera;
mod time_step;
mod game;
mod system;
mod buffer_resource;
mod inputs;
mod file_watcher;
mod utils;

use game::Game;
use renderer_context::{RendererContext, Resolution};

use system::System;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut renderer = RendererContext::new(
        &window, 
        Resolution { width: 800, height: 600 }
    ).await;

    let mut game = Game::new(&mut renderer);
    game.init(&mut renderer);
    
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            game.update();
            game.render(&mut renderer);
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                game.on_mouse_move(delta.0 as f32, delta.1 as f32);
            },
            DeviceEvent::MouseWheel { .. } => {
                //game.on_mouse_wheel(delta as f32);
            },
            _ => {}
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => {
                game.resize(&mut renderer, physical_size.width, physical_size.height);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                game.resize(&mut renderer, new_inner_size.width, new_inner_size.height);
            },
            WindowEvent::KeyboardInput { input, .. } => match input.state {
                ElementState::Pressed => {
                    if let Some(keycode) = input.virtual_keycode {
                        game.on_key_down(keycode);
                    }
                },
                ElementState::Released => {
                    if let Some(keycode) = input.virtual_keycode {
                        game.on_key_up(keycode);
                    }
                },
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    game.on_mouse_button_down(*button);
                },
                ElementState::Released => {
                    game.on_mouse_button_up(*button);
                },
            },
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    pollster::block_on(run());
}