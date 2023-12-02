use glam::Vec3;
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
        TextureHandle, Resolution}, 
    };

pub struct Game {
    inputs: Inputs,
    camera: Camera,
    globals: Globals,
    output_texture: TextureHandle,
    time_step: TimeStep,
    compute_pipeline: ComputePipelineHandle,
    render_pipeline: RenderPipelineHandle,
}

impl Game {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let inputs = Inputs::new();

        let camera = Camera::new(
            renderer,
            [0.0, 0.0, -1.0],
            1.0,
        );

        let globals = Globals::new(
            renderer, 
            800, 
            600,
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
    
        let render_shader_src = include_str!("shaders/render.wgsl");
        let render_shader = renderer.new_shader(render_shader_src);
    
        let render_pipeline = renderer.new_render_pipeline(
            &PipelineDesc {
                shader: render_shader,
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
        );
        
        let compute_shader_src = include_str!("shaders/compute.wgsl");
        let compute_shader = renderer.new_shader(compute_shader_src);
    
        let compute_pipeline = renderer.new_compute_pipeline(
            &PipelineDesc {
                shader: compute_shader,
                bindings_layout: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: globals.binding_type(),
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
        );

        Game {
            inputs,
            camera,
            globals,
            output_texture,
            time_step: TimeStep::new(),
            compute_pipeline,
            render_pipeline,
        }
    }
}

impl System for Game {
    fn init(&mut self, renderer: &mut RendererContext) {
        todo!()
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
    }

    fn render(&mut self, renderer: &mut RendererContext) {
        let compute_pass = ComputePass {
            pipeline: self.compute_pipeline,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::Texture(self.output_texture),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer(self.globals.get_buffer()),
                },
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer(self.camera.get_buffer()),
                },
            ],
        };

        let render_pass = RenderPass {
            pipeline: self.render_pipeline,
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

    fn on_mouse_move(&mut self, xDelta: f32, yDelta: f32) {
        self.inputs.on_mouse_move(xDelta, yDelta);
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
        self.globals.set_size(width, height);
        self.globals.update_buffer(renderer);
    }
}