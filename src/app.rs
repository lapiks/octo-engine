use std::sync::Arc;

use winit::{
    event::*, event_loop::EventLoop, keyboard::{Key, NamedKey}, window::{Window, WindowBuilder}
};

use crate::{game::Game, gui::{run_ui, EguiRenderer}, renderer_context::RendererContext, system::System};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

pub struct App {
    pub window: Arc<Window>,
    pub renderer: RendererContext,
    pub egui_renderer: EguiRenderer,
    pub game: Game,
}

impl App {
    async fn new(window: Arc<Window>) -> Self {
        let mut renderer = RendererContext::new(window.clone()).await;    
        let egui_renderer = EguiRenderer::new(&renderer, window.clone());
    
        let mut game = Game::new(&mut renderer);
        game.init(&mut renderer);

        Self {
            window,
            renderer,
            egui_renderer,
            game,
        }
    }

    pub async fn run() {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Octo Engine")
                .with_inner_size(winit::dpi::PhysicalSize {
                    width: INITIAL_WIDTH,
                    height: INITIAL_HEIGHT,
                })
                .build(&event_loop)
                .unwrap()
        );

        let mut app = pollster::block_on(App::new(window.clone()));
        
        let _ = event_loop.run(move |event, ewlt| match event {
            Event::WindowEvent { 
                window_id, 
                event 
            } if window_id == window.id() => {
                window.request_redraw();
                app.egui_renderer.handle_input(&window, &event);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        app.game.resize(&mut app.renderer, physical_size.width, physical_size.height);
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
                                    app.game.on_key_down(key);
                                },
                                winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                            }
                              
                        },
                        ElementState::Released => {
                            match event.physical_key {
                                winit::keyboard::PhysicalKey::Code(key) => {
                                    app.game.on_key_up(key);
                                },
                                winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                            }
                        },
                    },
                    WindowEvent::MouseInput { device_id, state, button } => match state {
                        ElementState::Pressed => {
                            app.game.on_mouse_button_down(button);
                        },
                        ElementState::Released => {
                            app.game.on_mouse_button_up(button);
                        },
                    },
                    WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                        //game.resize(&mut renderer, new_inner_size.width, new_inner_size.height);
                    },
                    WindowEvent::RedrawRequested => {
                        app.game.hot_reload(&mut app.renderer);
                        app.game.update();
                        app.game.prepare_rendering(&mut app.renderer);
    
                        if let Some(mut frame) =  app.renderer.begin_frame() {
                            app.game.render(&mut frame);
                            app.egui_renderer.render(
                                &app.renderer,
                                &mut frame, 
                                &window,
                                |ui| run_ui(ui)
                            );
                            app.renderer.commit_frame(frame);
                        }
                    },
                    _ => {}
                };
            }
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    app.game.on_mouse_move(delta.0 as f32, delta.1 as f32);
                },
                DeviceEvent::MouseWheel { delta } => {
                    match delta {
                        MouseScrollDelta::LineDelta(delta, _) => app.game.on_mouse_wheel(delta as f32),
                        MouseScrollDelta::PixelDelta(_) => todo!(),
                    }
                    
                },
                _ => {}
            }
            _ => {}
        });
    }
}