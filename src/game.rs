use std::{time::Duration, path::Path};

use glam::{Vec3, Vec2};
use thiserror::Error;
use winit::event::VirtualKeyCode;

use crate::{
    time_step::TimeStep, 
    system::System, 
    globals::Globals,
    camera::Camera, 
    inputs::Inputs,
    renderer_context::{
        RendererContext, 
        ComputePass, 
        Binding, 
        BindingResource, 
        RenderPass, 
        PipelineDesc, 
        ComputePipelineHandle, 
        RenderPipelineHandle, 
        TextureHandle, Resolution, ShaderHandle, RendererContextError}, 
    file_watcher::FileWatcher, 
    utils::make_relative_path, 
    voxel_world::VoxelWorld, 
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
    time_step: TimeStep,
    compute_shader: Option<ShaderHandle>,
    compute_pipeline: Option<ComputePipelineHandle>,
    render_shader: Option<ShaderHandle>,
    render_pipeline: Option<RenderPipelineHandle>,
    file_watcher: FileWatcher,
}

impl Game {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let file_watcher = Some(FileWatcher::new("./src/shaders", Duration::from_secs(5))).unwrap().unwrap();
        
        let world = VoxelWorld::new(renderer);
        
        let inputs = Inputs::new();

        let camera = Camera::new(
            renderer,
            [0.0, 0.0, -1.0],
            1.0,
        );

        let globals = Globals::new(
            renderer, 
            Vec2::new(800.0, 600.0)
        );

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
                format: wgpu::TextureFormat::Rgba8Uint,
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
            time_step: TimeStep::new(),
            compute_shader: None,
            compute_pipeline : None,
            render_shader: None,
            render_pipeline : None,
            file_watcher,
        }
    }

    fn create_shader<P: AsRef<Path>>(renderer: &mut RendererContext, path: P) -> Option<ShaderHandle> {
        let render_shader_src = std::fs::read_to_string(path).unwrap();
        match renderer.new_shader(render_shader_src.as_str()) {
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
                shader: shader,
                bindings_layout: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
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
        world: &VoxelWorld,
        globals: &Globals, 
        camera: &Camera
    ) -> ComputePipelineHandle{
        renderer.new_compute_pipeline(
            &PipelineDesc {
                shader: shader,
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
                            format: wgpu::TextureFormat::Rgba8Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: globals.binding_type(),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: camera.binding_type(),
                        count: None,
                    },
                ]
            }
        )
    }

    fn hot_reload(&mut self, renderer: &mut RendererContext) {
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
                                    self.compute_pipeline = Some(Game::create_compute_pipeline(renderer, compute_shader, &self.world, &self.globals, &self.camera))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl System for Game {
    fn init(&mut self, renderer: &mut RendererContext) {
        self.render_shader = Game::create_shader(renderer, "src/shaders/render.wgsl");
        self.compute_shader = Game::create_shader(renderer,"src/shaders/compute.wgsl");
        if let Some(render_shader) = self.render_shader {
            self.render_pipeline = Some(Game::create_render_pipeline(renderer, render_shader, &self.globals));
        }
        if let Some(compute_shader) = self.compute_shader {
            self.compute_pipeline = Some(Game::create_compute_pipeline(renderer, compute_shader, &self.world, &self.globals, &self.camera));
        }
    }

    fn update(&mut self) {
        let delta_time = self.time_step.tick();
        let speed = 2.0;

        if self.inputs.get_key_down(VirtualKeyCode::Z) {
            self.camera.translate(Vec3::Z * delta_time * speed);
        }
        if self.inputs.get_key_down(VirtualKeyCode::S) {
            self.camera.translate(Vec3::NEG_Z * delta_time * speed);
        }
        if self.inputs.get_key_down(VirtualKeyCode::Q) {
            self.camera.translate(Vec3::X * delta_time * speed);
        }
        if self.inputs.get_key_down(VirtualKeyCode::D) {
            self.camera.translate(Vec3::NEG_X * delta_time * speed);
        }

        self.inputs.reset();
    }

    fn render(&mut self, renderer: &mut RendererContext) {
        self.hot_reload(renderer);
        self.world.update_texture(renderer);

        if self.compute_pipeline.is_none() || self.render_pipeline.is_none() {
            return;
        }

        let compute_pass = ComputePass {
            pipeline: self.compute_pipeline.unwrap(),
            bindings: &[
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
                    resource: BindingResource::Buffer(self.globals.get_buffer()),
                },
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer(self.camera.get_buffer()),
                },
            ],
        };

        let render_pass = RenderPass {
            pipeline: self.render_pipeline.unwrap(),
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Texture(self.output_texture),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer(self.globals.get_buffer()),
                },
            ],
        };

        self.camera.update_buffer(renderer);
        renderer.commit_frame(&compute_pass, &render_pass);
    }

    fn on_key_down(&mut self, key: winit::event::VirtualKeyCode) {
        self.inputs.on_key_down(key);
    }

    fn on_key_up(&mut self, key: winit::event::VirtualKeyCode) {
        self.inputs.on_key_up(key);
    }

    fn on_mouse_button_down(&mut self, button: winit::event::MouseButton) {
        self.inputs.on_mouse_button_down(button);
    }

    fn on_mouse_button_up(&mut self, button: winit::event::MouseButton) {
        self.inputs.on_mouse_button_up(button);
    }

    fn on_mouse_move(&mut self, x_delta: f32, y_delta: f32) {
        self.inputs.on_mouse_move(x_delta, y_delta);
    }

    fn on_mouse_wheel(&mut self, delta: f32) {
        self.inputs.on_mouse_wheel(delta);
    }

    fn resize(&mut self, renderer: &mut RendererContext, width: u32, height: u32) {
        renderer.resize(Resolution { width: width, height: height });
        renderer.update_texture(
            self.output_texture,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: width,
                    height: height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Uint,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            }
        );
        self.globals.set_size(Vec2::new(width as f32, height as f32));
        self.globals.update_buffer(renderer);
    }
}