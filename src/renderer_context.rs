use slotmap::{SlotMap, new_key_type};
use thiserror::Error;
use wgpu::{BindGroupLayoutEntry, util::DeviceExt};

#[derive(Error, Debug)]
pub enum RendererContextError {
    #[error("Surface error")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Could not create shader module: {0}")]
    CreateShaderModule(String),
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Resolution {
    pub width: u32,
    pub height: u32
}

new_key_type! {
    pub struct TextureId;
    pub struct BufferId;
    pub struct ShaderId;
    pub struct RenderPipelineId;
    pub struct ComputePipelineId;
}

pub type TextureHandle = TextureId;
pub type BufferHandle = BufferId;
pub type ShaderHandle = ShaderId;
pub type RenderPipelineHandle = RenderPipelineId;
pub type ComputePipelineHandle = ComputePipelineId;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub struct PipelineDesc<'a> {
    pub shader: ShaderHandle,
    pub bindings_layout: &'a [BindGroupLayoutEntry],
}

pub struct ComputePass<'a> {
    pub bindings: &'a [Binding],
    pub pipeline: ComputePipelineHandle,
}

// impl<'a> ComputePass<'a> {
//     pub fn set_bindings(&mut self, bindings: &'a[Binding]) {
//         self.bindings = bindings;
//     }

//     pub fn set_pipeline(&mut self, pipeline: ComputePipelineHandle) {
//         self.pipeline = pipeline;
//     }

//     pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
//         //cpass.dispatch_workgroups(x, y, z);
//     }
// }

pub struct RenderPass<'a> {
    pub bindings: &'a [Binding],
    pub pipeline: RenderPipelineHandle,
}

// impl<'a> RenderPass<'a> {
//     pub fn set_bind_group(&mut self, bindings: &'a[Binding]) {
//         self.bindings = bindings;
//     }

//     pub fn set_pipeline(&mut self, pipeline: RenderPipelineHandle) {
//         self.pipeline = pipeline;
//     }

//     pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {
//         //cpass.dispatch_workgroups(x, y, z);
//     }
// }

pub enum BindingResource {
    Texture(TextureId),
    Buffer(BufferId),
}

pub struct Binding {
    pub binding: u32,
    pub resource: BindingResource,
}

pub struct RendererContext {
    surface: wgpu::Surface,
    surface_conf: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resolution: Resolution,
    textures: SlotMap<TextureId, wgpu::TextureView>,
    buffers: SlotMap<BufferId, wgpu::Buffer>,
    shaders: SlotMap<ShaderId, wgpu::ShaderModule>,
    render_pipelines: SlotMap<RenderPipelineId, RenderPipeline>,
    compute_pipelines: SlotMap<ComputePipelineId, ComputePipeline>,
}

impl RendererContext {
    pub async fn new<
    W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
>(window: &W, resolution: Resolution) -> Self {
        env_logger::init();

        // Instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: Default::default(),
            gles_minor_version: Default::default(),
        });
        
        // Surface
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        
        // Adapter
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // Device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())            
            .unwrap_or(surface_caps.formats[0]);

        let surface_conf = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: resolution.width,
            height: resolution.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_conf);

        Self {
            surface,
            surface_conf,
            device,
            queue,
            resolution,
            textures: SlotMap::default(),
            buffers: SlotMap::default(),
            shaders: SlotMap::default(),
            render_pipelines: SlotMap::default(),
            compute_pipelines: SlotMap::default(),
        }
    }

    pub fn resize(&mut self, resolution: Resolution) {
        if resolution.width > 0 && resolution.height > 0 {
            self.resolution = resolution;
            self.surface_conf.width = resolution.width;
            self.surface_conf.height = resolution.height;
            self.surface.configure(&self.device, &self.surface_conf);
        }
    }

    pub fn new_shader(&mut self, src: &str) -> Result<ShaderHandle, RendererContextError> {
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
        let error = self.device.pop_error_scope();

        if let Some(wgpu::Error::Validation { description, .. }) = pollster::block_on(error) {
            return Err(RendererContextError::CreateShaderModule(description));
        }
        
        Ok(self.shaders.insert(shader))
    }

    pub fn destroy_shader(&mut self, handle: ShaderHandle) {
        self.shaders.remove(handle);
    }

    pub fn new_render_pipeline(&mut self, desc: &PipelineDesc) -> RenderPipelineHandle {
        let shader_module: &wgpu::ShaderModule = self.shaders.get(desc.shader).unwrap(); // todo: remove unwrap

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: desc.bindings_layout,
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(self.surface_conf.format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        self.render_pipelines.insert(
            RenderPipeline {
                pipeline: render_pipeline,
                bind_group_layout
            }
        )
    }

    pub fn destroy_render_pipeline(&mut self, handle: RenderPipelineHandle) {
        self.render_pipelines.remove(handle);
    }

    pub fn new_compute_pipeline(&mut self, desc: &PipelineDesc) -> ComputePipelineHandle {
        let shader_module: &wgpu::ShaderModule = self.shaders.get(desc.shader).unwrap(); // todo: remove unwrap

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: desc.bindings_layout,
        });
    
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
        });

        self.compute_pipelines.insert(
            ComputePipeline {
                pipeline: compute_pipeline,
                bind_group_layout,
            }
        )
    }

    pub fn destroy_compute_pipeline(&mut self, handle: ComputePipelineHandle) {
        self.compute_pipelines.remove(handle);
    }

    pub fn new_buffer(&mut self, desc: &wgpu::util::BufferInitDescriptor) -> BufferHandle {
        let globals_buffer = self.device.create_buffer_init(desc);

        self.buffers.insert(
            globals_buffer
        )
    }

    pub fn destroy_buffer(&mut self, handle: BufferHandle) {
        self.buffers.remove(handle);
    }

    pub fn update_buffer(&mut self, handle: BufferHandle, contents: &[u8]) {
        if let Some(buffer) = self.buffers.get_mut(handle) {
            self.queue.write_buffer(buffer, 0, contents);
        }
        else {
            print!("Unknown buffer")
        }
    }

    pub fn new_texture(&mut self, desc: &wgpu::TextureDescriptor) -> TextureHandle {
        let texture = self.device.create_texture(desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.textures.insert(
            texture_view
        )
    }

    pub fn destroy_texture(&mut self, handle: TextureHandle) {
        self.textures.remove(handle);
    }

    pub fn update_texture(&mut self, handle: TextureHandle, desc: &wgpu::TextureDescriptor) {
        if let Some(texture_view) = self.textures.get_mut(handle) {
            let texture = self.device.create_texture(desc);
            *texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
        else {
            print!("Unknown texture")
        }
    }

    pub fn commit_frame(&mut self, compute_pass: &ComputePass, render_pass: &RenderPass) -> Result<(), wgpu::SurfaceError> {
        let render_pipeline_id = render_pass.pipeline;        
        let render_pipeline = self.render_pipelines.get(render_pipeline_id).unwrap();
        let compute_pipeline_id = compute_pass.pipeline;     
        let compute_pipeline = self.compute_pipelines.get(compute_pipeline_id).unwrap();

        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });
        
                // Compute pass
                {
                    let mut bindings = Vec::with_capacity(compute_pass.bindings.len());
                    for binding in compute_pass.bindings {
                        match binding.resource {
                            BindingResource::Texture(id) => {
                                if let Some(texture) = self.textures.get(id) {
                                    bindings.push(
                                        wgpu::BindGroupEntry {
                                            binding: binding.binding,
                                            resource: wgpu::BindingResource::TextureView(texture),
                                        }
                                    )
                                }
                            },
                            BindingResource::Buffer(id) => {
                                if let Some(buffer) = self.buffers.get(id) {
                                    bindings.push(
                                        wgpu::BindGroupEntry {
                                            binding: binding.binding,
                                            resource: buffer.as_entire_binding(),
                                        }
                                    )
                                }
                            },
                        };
                    }

                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &compute_pipeline.bind_group_layout,
                        entries: &bindings,
                        label: None,
                    });

                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Compute pass"),
                        timestamp_writes: None,
                    });

                    cpass.set_bind_group(0, &bind_group, &[]);
                    cpass.set_pipeline(&compute_pipeline.pipeline);
                    cpass.dispatch_workgroups(self.resolution.width, self.resolution.height, 1); // Number of cells to run, the (x,y,z) size of item being processed
                }

                // Render pass 
                {
                    let mut bindings = Vec::with_capacity(render_pass.bindings.len());
                    for binding in render_pass.bindings {
                        match binding.resource {
                            BindingResource::Texture(id) => {
                                if let Some(texture) = self.textures.get(id) {
                                    bindings.push(
                                        wgpu::BindGroupEntry {
                                            binding: binding.binding,
                                            resource: wgpu::BindingResource::TextureView(texture),
                                        }
                                    )
                                }
                            },
                            BindingResource::Buffer(id) => {
                                if let Some(buffer) = self.buffers.get(id) {
                                    bindings.push(
                                        wgpu::BindGroupEntry {
                                            binding: binding.binding,
                                            resource: buffer.as_entire_binding(),
                                        }
                                    )
                                }
                            },
                        };
                    }

                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &render_pipeline.bind_group_layout,
                        entries: &bindings,
                    });

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        label: Some("Render pass"),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
        
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_pipeline(&render_pipeline.pipeline);
                    render_pass.draw(0..3, 0..1);
                }
            
                // submit will accept anything that implements IntoIter
                self.queue.submit(std::iter::once(encoder.finish()));
                frame.present();
            }
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => self.resize(self.resolution),
            // The system is out of memory, we should probably quit
            //Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }        
    
        Ok(())
    }
}