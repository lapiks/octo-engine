use std::sync::Arc;

use winit::{
    event::*, event_loop::EventLoop, keyboard::{Key, KeyCode, NamedKey}, window::{Window, WindowBuilder}
};

use crate::{editor::Editor, egui_renderer::EguiRenderer, game::Game, renderer_context::{RendererContext, Resolution}, system::System};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

pub struct App {
    pub window: Arc<Window>,
    pub game: Game,
    pub editor: Editor,
    pub show_editor: bool,
    pub viewport_size: Resolution,
}

impl App {
    async fn new(window: Arc<Window>, renderer: &mut RendererContext) -> Self {   
        let mut game = Game::new(renderer);
        game.init(renderer);

        Self {
            window,
            game,
            editor: Editor::new(),
            show_editor: true,
            viewport_size: Resolution {
                width: INITIAL_WIDTH,
                height: INITIAL_HEIGHT,
            } 
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

        let mut renderer = RendererContext::new(window.clone()).await;    

        let mut app = pollster::block_on(App::new(window.clone(), &mut renderer));
        let mut egui_renderer = EguiRenderer::new(&renderer, window.clone());

        let _ = event_loop.run(move |event, ewlt| match event {
            Event::WindowEvent { 
                window_id, 
                event 
            } if window_id == window.id() => {
                window.request_redraw();
                egui_renderer.handle_input(&window, &event);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        if !app.show_editor {
                            app.viewport_size.width = physical_size.width;
                            app.viewport_size.height = physical_size.height;
                        }
                        renderer.resize(
                            Resolution {
                                width: physical_size.width,
                                height: physical_size.height,
                            }
                        );
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
                                    if key == KeyCode::F1 {
                                        app.show_editor = !app.show_editor;
                                    }
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
                        app.game.hot_reload(&mut renderer);
                        app.game.update();

                        app.game.resize(
                            &mut renderer, 
                            match app.show_editor {
                                true => {
                                    let rect = app.editor.viewport_rect;
                                    Resolution {
                                        width: rect.width() as u32,
                                        height: rect.height() as u32,
                                    }
                                }
                                false => app.viewport_size,
                            }
                        );

                        app.game.prepare_rendering(&mut renderer);

                        if let Some(mut frame) = renderer.begin_frame() {
                            app.game.render(&mut frame);
                            if app.show_editor {
                                let game_texture = egui_renderer.register_native_texture(&renderer, app.game.game_texture());
                                egui_renderer.render(
                                    &renderer,
                                    &mut frame, 
                                    &window,
                                    |ui| app.run_ui(ui, &renderer, game_texture)
                                );
                            }
                            renderer.commit_frame(frame);
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

    pub fn run_ui(&mut self, ctx: &egui::Context, renderer: &RendererContext, game_texture: Option<egui::TextureId>) {
        self.editor.run_ui(ctx, &self.game, renderer, game_texture);
    }
}