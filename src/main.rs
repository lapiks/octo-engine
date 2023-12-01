mod renderer_context;
mod globals;
mod camera;
mod time_step;
mod game;
mod system;

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
    
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            game.render(&mut renderer);
        }
        Event::MainEventsCleared => {
            window.request_redraw();
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
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(k) => {
                    game.on_key_down(k);
                }
                None => todo!(),
            },
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}

fn main() {
    pollster::block_on(run());
}