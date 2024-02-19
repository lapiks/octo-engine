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
mod transform;

use game::Game;
use gui::{run_ui, EguiRenderer};
use renderer_context::{RendererContext, Resolution};

use system::System;
use winit::{
    event::*, event_loop::EventLoop, keyboard::{Key, NamedKey}, window::WindowBuilder
};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
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

    
    let mut gui_renderer = EguiRenderer::new(&renderer, &window);
    let mut game = Game::new(&mut renderer);
    game.init(&mut renderer);
    
    let _ = event_loop.run(move |event, ewlt| match event {
        Event::WindowEvent { 
            window_id, 
            event 
        } if window_id == window.id() => {
            window.request_redraw();
            gui_renderer.handle_input(&window, &event);
            match event {
                WindowEvent::Resized(physical_size) => {
                    game.resize(&mut renderer, physical_size.width, physical_size.height);
                },
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                    ..
                } => ewlt.exit(),
                WindowEvent::KeyboardInput { device_id, event, is_synthetic } => match event.state {
                    ElementState::Pressed => {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(key) => {
                                game.on_key_down(key);
                            },
                            winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                        }
                          
                    },
                    ElementState::Released => {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(key) => {
                                game.on_key_up(key);
                            },
                            winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                        }
                    },
                },
                WindowEvent::MouseInput { device_id, state, button } => match state {
                    ElementState::Pressed => {
                        game.on_mouse_button_down(button);
                    },
                    ElementState::Released => {
                        game.on_mouse_button_up(button);
                    },
                },
                WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                    //game.resize(&mut renderer, new_inner_size.width, new_inner_size.height);
                },
                WindowEvent::RedrawRequested => {
                    game.hot_reload(&mut renderer);
                    game.update();
                    game.prepare_rendering(&mut renderer);

                    if let Some(mut frame) =  renderer.begin_frame() {
                        game.render(&mut frame);
                        gui_renderer.render(
                            &renderer,
                            &mut frame, 
                            &window,
                            |ui| run_ui(ui)
                        );
                        renderer.commit_frame(frame);
                    }
                },
                _ => {}
            };
        }
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => {
                game.on_mouse_move(delta.0 as f32, delta.1 as f32);
            },
            DeviceEvent::MouseWheel { delta } => {
                match delta {
                    MouseScrollDelta::LineDelta(delta, _) => game.on_mouse_wheel(delta as f32),
                    MouseScrollDelta::PixelDelta(_) => todo!(),
                }
                
            },
            _ => {}
        }
        _ => {}
    });
}

fn main() {
    pollster::block_on(run());
}