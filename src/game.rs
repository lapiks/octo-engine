use std::{time::Duration, path::Path};

use glam::{vec2, vec3, Vec2, Vec3};
use thiserror::Error;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    camera::Camera, file_watcher::FileWatcher, globals::Globals, inputs::Inputs, renderer_context::{
        BindGroupHandle, Binding, BindingResource, ComputePassDesc, ComputePipelineHandle, Frame, PipelineDesc, RenderPassDesc, RenderPipelineHandle, RendererContext, RendererContextError, Resolution, ShaderHandle, TextureHandle
    }, sprite::Sprite, system::System, utils::make_relative_path, voxel_world::VoxelWorld 
};


#[derive(Error, Debug)]
pub enum GameError {
    #[error("Renderer Context error")]
    RendererContextError(#[from] RendererContextError),
}


pub struct Game {
    world: VoxelWorld,
    inputs: Inputs,
    camera: Camera,
    globals: Globals,
    output_texture: TextureHandle,
    compute_shader: Option<ShaderHandle>,
    compute_pipeline: Option<ComputePipelineHandle>,
    compute_bind_group: Option<BindGroupHandle>,
    render_shader: Option<ShaderHandle>,
    render_pipeline: Option<RenderPipelineHandle>,
    render_bind_group: Option<BindGroupHandle>,
    sprites: Vec<Sprite>,
    file_watcher: FileWatcher,
}

impl Game {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let file_watcher = Some(
            FileWatcher::new(
                "./src/shaders", 
                Duration::from_secs(5)
            )
        ).unwrap().unwrap();

        let inputs = Inputs::new();
        
        // game datas
        let world = VoxelWorld::new(renderer);
        let mut camera = Camera::new(
            renderer,
            vec2(800.0, 600.0),
        );

        //camera.transform.position = vec3(8.0, 0.0, 8.0);

        let globals = Globals::new(renderer);

        // render texture
        let output_texture = renderer.new_texture(
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 800,
                    height: 600,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            }
        );

        Game {
            world,
            inputs,
            camera,
            globals,
            output_texture,
            compute_shader: None,
            compute_pipeline : None,
            compute_bind_group: None,
            render_shader: None,
            render_pipeline : None,
            render_bind_group: None,
            sprites: vec![],
            file_watcher,
        }
    }

    fn create_shader<P: AsRef<Path>>(renderer: &mut RendererContext, path: P) -> Option<ShaderHandle> {
        let shader_src = std::fs::read_to_string(path).unwrap();
        match renderer.new_shader(shader_src.as_str()) {
            Ok(shader) => {
                return Some(shader);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }

        None
    }

    fn create_render_pipeline(renderer: &mut RendererContext, shader: ShaderHandle, globals: &Globals) -> RenderPipelineHandle {
        renderer.new_render_pipeline(
            &PipelineDesc {
                shader,
                bindings_layout: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: globals.binding_type(),
                        count: None,
                    },
                ]
            }
        )
    }

    fn create_compute_pipeline(
        renderer: &mut RendererContext, 
        shader: ShaderHandle, 
        globals: &Globals, 
        camera: &Camera
    ) -> ComputePipelineHandle{
        renderer.new_compute_pipeline(
            &PipelineDesc {
                shader,
                bindings_layout: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: VoxelWorld::binding_type(),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: camera.binding_type(),
                        count: None,
                    },
                ]
            }
        )
    }

    pub fn hot_reload(&mut self, renderer: &mut RendererContext) {
        if let Some(watcher_event) = self.file_watcher.get_event() {
            if let notify::EventKind::Modify(_) = watcher_event.kind {
                for path in watcher_event.paths.iter() {
                    if let Ok(relative_path) = make_relative_path(path) {
                        if let Some(file_stem) = relative_path.file_stem() {
                            if file_stem == "render" {
                                if let Some(render_shader) = self.render_shader {
                                    renderer.destroy_shader(render_shader);
                                }
                                if let Some(render_pipeline) = self.render_pipeline {
                                    renderer.destroy_render_pipeline(render_pipeline);
                                }
                                self.render_shader = Game::create_shader(renderer, "src/shaders/render.wgsl");
                                if let Some(render_shader) = self.render_shader {
                                    self.render_pipeline = Some(Game::create_render_pipeline(renderer, render_shader, &self.globals))
                                }
                            }
                            else if file_stem == "compute" {
                                if let Some(compute_shader) = self.compute_shader {
                                    renderer.destroy_shader(compute_shader);
                                }
                                if let Some(compute_pipeline) = self.compute_pipeline {
                                    renderer.destroy_compute_pipeline(compute_pipeline);
                                }
                                self.compute_shader = Game::create_shader(renderer, "src/shaders/compute.wgsl");
                                if let Some(compute_shader) = self.compute_shader {
                                    self.compute_pipeline = Some(Game::create_compute_pipeline(renderer, compute_shader, &self.globals, &self.camera))
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn game_texture(&self) -> TextureHandle {
        self.output_texture
    }
}

impl System for Game {
    fn init(&mut self, renderer: &mut RendererContext) {
        self.render_shader = Game::create_shader(renderer, "src/shaders/quad_renderer.wgsl");
        self.render_pipeline = self.render_shader
            .and_then(|shader| {
                Some(Game::create_render_pipeline(
                    renderer, 
                    shader, 
                    &self.globals
                ))
            });
        self.compute_shader = Game::create_shader(renderer,"src/shaders/voxel_renderer.wgsl");
        self.compute_pipeline = self.compute_shader
            .and_then(|shader| {
                Some(Game::create_compute_pipeline(
                    renderer, 
                    shader, 
                    &self.globals, 
                    &self.camera
                ))
            });

        self.camera.transform.position = Vec3::new(32.0, 16.0, 32.0);
        self.camera.transform.look_at(vec3(16.0, 16.0, 16.0), Vec3::Y);

        self.sprites.push(Sprite::new(renderer, &self.world));
    }

    fn update(&mut self, delta_time: f32) {
        let speed = 25.0;
        let rot_speed = 10.0;

        if self.inputs.get_key_down(KeyCode::KeyW) {
            self.camera.transform.translate(self.camera.transform.forward() * delta_time * speed);
        }
        if self.inputs.get_key_down(KeyCode::KeyS) {
            self.camera.transform.translate(self.camera.transform.back() * delta_time * speed);
        }
        if self.inputs.get_key_down(KeyCode::KeyD) {
            self.camera.transform.translate(self.camera.transform.right() * delta_time * speed);
        }
        if self.inputs.get_key_down(KeyCode::KeyA) {
            self.camera.transform.translate(self.camera.transform.left() * delta_time * speed);
        }
        if self.inputs.get_key_down(KeyCode::Space) {
            self.camera.transform.translate(Vec3::Y * delta_time * speed);
        }
        if self.inputs.get_key_down(KeyCode::ControlLeft) {
            self.camera.transform.translate(Vec3::NEG_Y * delta_time * speed);
        }
        if self.inputs.get_key_down(KeyCode::KeyE) {
            self.camera.transform.rotate_y(delta_time * rot_speed);
        }
        if self.inputs.get_key_down(KeyCode::KeyQ) {
            self.camera.transform.rotate_y(-delta_time * rot_speed);
        }

        self.inputs.reset();
    }

    /// Prepare resources for rendering
    fn prepare_rendering(&mut self, renderer: &mut RendererContext) {
        self.camera.update_buffer(renderer);
        self.world.update_texture(renderer);
        
        if let Some(compute_bind_group) = self.compute_bind_group {
            renderer.destroy_bind_group(compute_bind_group);
        }
        
        self.compute_bind_group = Some(renderer.new_compute_bind_group(
            self.compute_pipeline.unwrap(), 
            &[
            Binding {
                binding: 0,
                resource: BindingResource::Texture(self.world.get_texture()),
            },
            Binding {
                binding: 1,
                resource: BindingResource::Texture(self.output_texture),
            },
            Binding {
                binding: 2,
                resource: BindingResource::Buffer(self.camera.get_buffer()),
            }]
        ));

        if let Some(render_bind_group) = self.render_bind_group {
            renderer.destroy_bind_group(render_bind_group);
        }

        self.render_bind_group = Some(renderer.new_render_bind_group(
            self.render_pipeline.unwrap(), 
            &[
            Binding {
                binding: 0,
                resource: BindingResource::Texture(self.output_texture),
            },
            Binding {
                binding: 1,
                resource: BindingResource::Buffer(self.camera.get_buffer()),
            }]
        ));
    }

    fn render(&mut self, frame: &mut Frame) {
        for sprite in &self.sprites {
            let mut cpass = frame.begin_compute_pass(
                &ComputePassDesc {
                pipeline: sprite.compute_pipeline,
                bind_group: sprite.compute_bind_group,
            });
            cpass.dispatch(32, 32, 32);
        }

        // Compute pass
        {
            let mut cpass = frame.begin_compute_pass(
                &ComputePassDesc {
                pipeline: self.compute_pipeline.unwrap(),
                bind_group: self.compute_bind_group.unwrap(),
            });
            let output_size = self.camera.size();
            cpass.dispatch(output_size.x as u32, output_size.y as u32, 1);
        }

        // Render pass
        {
            let mut rpass = frame.begin_render_pass(
                &RenderPassDesc {
                pipeline: self.render_pipeline.unwrap(),
                bind_group: self.render_bind_group.unwrap(),
                load_op: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
            });
            rpass.draw(0..3, 0..1);
        }
    }

    fn on_key_down(&mut self, key: KeyCode) {
        self.inputs.on_key_down(key);
    }

    fn on_key_up(&mut self, key: KeyCode) {
        self.inputs.on_key_up(key);
    }

    fn on_mouse_button_down(&mut self, button: MouseButton) {
        self.inputs.on_mouse_button_down(button);
    }

    fn on_mouse_button_up(&mut self, button: MouseButton) {
        self.inputs.on_mouse_button_up(button);
    }

    fn on_mouse_move(&mut self, x_delta: f32, y_delta: f32) {
        self.inputs.on_mouse_move(x_delta, y_delta);
    }

    fn on_mouse_wheel(&mut self, delta: f32) {
        self.inputs.on_mouse_wheel(delta);
    }

    fn resize(&mut self, renderer: &mut RendererContext, resolution: Resolution) {
        if resolution.width == 0 || resolution.height == 0 {
            return;
        }

        renderer.update_texture(
            self.output_texture,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: resolution.width,
                    height: resolution.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            }
        );
        self.camera.set_size(Vec2::new(resolution.width as f32, resolution.height as f32));
        self.camera.update_buffer(renderer);
    }
}