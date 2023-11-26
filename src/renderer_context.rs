use slotmap::{SlotMap, new_key_type};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererContextError {
    #[error("Surface error")]
    SurfaceError(#[from] wgpu::SurfaceError),
}

#[derive(Clone, Copy)]
pub struct Resolution {
    pub width: u32,
    pub height: u32
}

new_key_type! {
    pub struct ShaderId;
    pub struct RenderPipelineId;
    pub struct ComputePipelineId;
}

type ShaderHandle = ShaderId;
type RenderPipelineHandle = RenderPipelineId;
type ComputePipelineHandle = ComputePipelineId;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
}

pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
}

pub struct RendererContext {
    surface: wgpu::Surface,
    surface_conf: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resolution: Resolution,
    shaders: SlotMap<ShaderId, wgpu::ShaderModule>,
    render_pipelines: SlotMap<RenderPipelineId, RenderPipeline>,
    compute_pipelines: SlotMap<ComputePipelineId, ComputePipeline>,
    current_render_pipeline: Option<RenderPipelineHandle>,
    current_compute_pipeline: Option<ComputePipelineHandle>,
    storage_texture_view: wgpu::TextureView,
}

pub struct RenderPipelineDesc {
    pub shader: ShaderHandle,
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

        let storage_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: resolution.width,
                height: resolution.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let storage_texture_view = storage_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Self {
            surface,
            surface_conf,
            device,
            queue,
            resolution,
            shaders: SlotMap::default(),
            render_pipelines: SlotMap::default(),
            compute_pipelines: SlotMap::default(),
            current_render_pipeline: None,
            current_compute_pipeline: None,
            storage_texture_view,
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

    pub fn new_shader(&mut self, src: &str) -> ShaderHandle {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(src.into()),
        });
        
        self.shaders.insert(shader)
    }

    pub fn new_render_pipeline(&mut self, desc: &RenderPipelineDesc) -> RenderPipelineHandle {
        let shader_module: &wgpu::ShaderModule = self.shaders.get(desc.shader).unwrap(); // todo: remove unwrap

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
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
            ],
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.storage_texture_view),
                },
            ],
            label: None,
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
                bind_group
            }
        )
    }

    pub fn new_compute_pipeline(&mut self, desc: &RenderPipelineDesc) -> ComputePipelineHandle {
        let shader_module: &wgpu::ShaderModule = self.shaders.get(desc.shader).unwrap(); // todo: remove unwrap

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.storage_texture_view),
                }
            ],
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
                bind_group,
            }
        )
    }

    pub fn set_render_pipeline(&mut self, render_pipeline: RenderPipelineHandle) {
        self.current_render_pipeline = Some(render_pipeline);
    }

    pub fn set_compute_pipeline(&mut self, compute_pipeline: ComputePipelineHandle) {
        self.current_compute_pipeline = Some(compute_pipeline);
    }

    pub fn commit_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_pipeline_id = self.current_render_pipeline.unwrap();        
        let render_pipeline = self.render_pipelines.get(render_pipeline_id).unwrap();
        let compute_pipeline_id = self.current_compute_pipeline.unwrap();     
        let compute_pipeline = self.compute_pipelines.get(compute_pipeline_id).unwrap();

        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });
        
                // Compute pass
                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Compute pass"),
                        timestamp_writes: None,
                    });

                    cpass.set_bind_group(0, &compute_pipeline.bind_group, &[]);
                    cpass.set_pipeline(&compute_pipeline.pipeline);
                    cpass.dispatch_workgroups(self.resolution.width, self.resolution.height, 1); // Number of cells to run, the (x,y,z) size of item being processed
                }

                // Render pass 
                {
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
        
                    render_pass.set_bind_group(0, &render_pipeline.bind_group, &[]);
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