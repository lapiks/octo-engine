mod renderer_context;
mod globals;
mod camera;
mod time_step;

use camera::Camera;
use globals::Globals;
use renderer_context::{RendererContext, Resolution, PipelineDesc, ComputePass, RenderPass, Binding, BindingResource};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut globals = Globals {
        width: 800,
        height: 600,
    };

    let mut camera = Camera {
        position: [0.0, 0.0, -1.0],
        focal_length: 1.0,
    };

    let mut ctx = RendererContext::new(
        &window, 
        Resolution { width: globals.width, height: globals.height }
    ).await;

    let output_texture = ctx.new_texture(
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: globals.width,
                height: globals.height,
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

    let globals_buffer = ctx.new_buffer(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Globals buffer"),
            contents: bytemuck::bytes_of(&globals),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let camera_buffer = ctx.new_buffer(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Globals buffer"),
            contents: bytemuck::bytes_of(&camera),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let render_shader_src = include_str!("shaders/render.wgsl");
    let render_shader = ctx.new_shader(render_shader_src);

    let render_pipeline = ctx.new_render_pipeline(
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
    let compute_shader = ctx.new_shader(compute_shader_src);

    let compute_pipeline = ctx.new_compute_pipeline(
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
    
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            let compute_pass = ComputePass {
                pipeline: compute_pipeline,
                bindings: &[
                    Binding {
                        binding: 0,
                        resource: BindingResource::Texture(output_texture),
                    },
                    Binding {
                        binding: 1,
                        resource: BindingResource::Buffer(globals_buffer),
                    },
                    Binding {
                        binding: 2,
                        resource: BindingResource::Buffer(camera_buffer),
                    },
                ],
            };

            let render_pass = RenderPass {
                pipeline: render_pipeline,
                bindings: &[
                    Binding {
                        binding: 0,
                        resource: BindingResource::Texture(output_texture),
                    },
                    Binding {
                        binding: 1,
                        resource: BindingResource::Buffer(globals_buffer),
                    },
                ],
            };

            ctx.update_buffer(camera_buffer, bytemuck::bytes_of(&camera));

            ctx.commit_frame(&compute_pass, &render_pass);
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => {
                ctx.resize(Resolution { width: physical_size.width, height: physical_size.height });
                ctx.update_texture(
                    output_texture,
                    &wgpu::TextureDescriptor {
                        label: None,
                        size: wgpu::Extent3d {
                            width: physical_size.width,
                            height: physical_size.height,
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
                globals.width = physical_size.width;
                globals.height = physical_size.height;
                ctx.update_buffer(
                    globals_buffer, 
                    bytemuck::bytes_of(&globals)
                );
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                ctx.resize(Resolution { width: new_inner_size.width, height: new_inner_size.height });
            },
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(k) => match k {
                    winit::event::VirtualKeyCode::Z => {
                        camera.position[2] += 1.0;
                    }
                    winit::event::VirtualKeyCode::S => {
                        camera.position[2] -= 1.0;
                    }
                    winit::event::VirtualKeyCode::Q => {
                        camera.position[0] -= 1.0;
                    }
                    winit::event::VirtualKeyCode::D => {
                        camera.position[0] += 1.0;
                    }
                    _ => (),
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