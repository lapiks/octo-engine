mod renderer_context;
mod globals;
mod camera;
mod time_step;
mod game;
mod system;
mod inputs;
mod file_watcher;
mod utils;
mod voxel_world;
mod gui;
mod ray;
mod color;

use game::Game;
use gui::Gui;
use renderer_context::{RendererContext, Resolution};

use system::System;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Octo Engine")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let mut renderer = RendererContext::new(
        &window, 
        Resolution { width: INITIAL_WIDTH, height: INITIAL_HEIGHT }
    ).await;

    
    let mut gui = Gui::new(&renderer);
    let mut game = Game::new(&mut renderer);
    game.init(&mut renderer);
    
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            game.hot_reload(&mut renderer);
            game.update();
            game.prepare_rendering(&mut renderer);

            if let Some(mut frame) =  renderer.begin_frame() {
                game.render(&mut frame);
                gui.render(&mut frame);
                renderer.commit_frame(frame);
            }
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