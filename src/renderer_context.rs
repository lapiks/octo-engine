use std::{ops::Range, sync::Arc};

use slotmap::{SlotMap, new_key_type};
use thiserror::Error;
use wgpu::{util::DeviceExt, BindGroupLayoutEntry, Color, Extent3d, ImageDataLayout};
use winit::window::Window;

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
    pub struct BindGroupId;
}

pub type TextureHandle = TextureId;
pub type BufferHandle = BufferId;
pub type ShaderHandle = ShaderId;
pub type RenderPipelineHandle = RenderPipelineId;
pub type ComputePipelineHandle = ComputePipelineId;
pub type BindGroupHandle = BindGroupId;

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

pub struct RenderPass<'a> {
    pub pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    pub fn new(pass: wgpu::RenderPass<'a>) -> Self {
        Self {
            pass
        }
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.pass.draw(vertices, instances);
    }
}

pub struct ComputePass<'a> {
    pass: wgpu::ComputePass<'a>,
}

impl<'a> ComputePass<'a> {
    pub fn new(pass: wgpu::ComputePass<'a>) -> Self {
        Self {
            pass
        }
    }

    pub fn dispatch(& mut self, x: u32, y: u32, z: u32) {
        self.pass.dispatch_workgroups(x, y, z);
    }
}

pub struct ComputePassDesc {
    pub bind_group: BindGroupHandle,
    pub pipeline: ComputePipelineHandle,
}
pub struct RenderPassDesc {
    pub bind_group: BindGroupHandle,
    pub pipeline: RenderPipelineHandle,
    pub load_op: wgpu::LoadOp<Color>,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView, 
}

pub enum BindingResource {
    Texture(TextureId),
    Buffer(BufferId),
}

pub struct Binding {
    pub binding: u32,
    pub resource: BindingResource,
}

/// Permits render to the current surface texture
pub struct Frame<'a> {
    renderer: &'a RendererContext,
    surface_texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'a> Frame<'a> {
    pub fn new(
        renderer: &'a RendererContext, 
        surface_texture: wgpu::SurfaceTexture,
        view: wgpu::TextureView, 
        encoder: wgpu::CommandEncoder) -> Self {
        Self {
            renderer,
            surface_texture,
            view,
            encoder,
        }
    }

    pub fn new_render_pass(&mut self, load_op: wgpu::LoadOp<wgpu::Color>) -> wgpu::RenderPass {
        self.encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("Render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            }
        )
    }

    pub fn begin_render_pass(&mut self, desc: &RenderPassDesc) -> RenderPass {
        let render_pipeline = self.renderer.render_pipelines.get(desc.pipeline).unwrap();
        let bind_group = self.renderer.bind_groups.get(desc.bind_group).unwrap();

        let mut render_pass = self.new_render_pass(desc.load_op);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_pipeline(&render_pipeline.pipeline);

        RenderPass::new(render_pass)
    }

    pub fn begin_compute_pass(&mut self, desc: &ComputePassDesc) -> ComputePass {
        let compute_pipeline = self.renderer.compute_pipelines.get(desc.pipeline).unwrap();
        let bind_group = self.renderer.bind_groups.get(desc.bind_group).unwrap();

        let mut cpass = self.encoder.begin_compute_pass(
            &wgpu::ComputePassDescriptor {
                label: Some("Compute pass"),
                timestamp_writes: None,
            }
        );

        cpass.set_bind_group(0, bind_group, &[]);
        cpass.set_pipeline(&compute_pipeline.pipeline);
        
        ComputePass::new(cpass)
    }

    pub fn encoder_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}

pub struct RendererContext {
    surface: wgpu::Surface<'static>,
    surface_conf: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    resolution: Resolution,
    textures: SlotMap<TextureId, Texture>,
    buffers: SlotMap<BufferId, wgpu::Buffer>,
    shaders: SlotMap<ShaderId, wgpu::ShaderModule>,
    render_pipelines: SlotMap<RenderPipelineId, RenderPipeline>,
    compute_pipelines: SlotMap<ComputePipelineId, ComputePipeline>,
    bind_groups: SlotMap<BindGroupId, wgpu::BindGroup>,
}

impl RendererContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        env_logger::init();

        // Instance
        let instance = wgpu::Instance::default();
        
        // Surface
        let surface = instance.create_surface(window.clone()).unwrap();
        
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
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_conf = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &surface_conf);

        Self {
            surface,
            surface_conf,
            device,
            queue,
            resolution: Resolution {
                width: size.width,
                height: size.height,
            },
            textures: SlotMap::default(),
            buffers: SlotMap::default(),
            shaders: SlotMap::default(),
            render_pipelines: SlotMap::default(),
            compute_pipelines: SlotMap::default(),
            bind_groups: SlotMap::default(),
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
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

    pub fn new_render_bind_group(&mut self, pipeline: RenderPipelineHandle, bindings: &[Binding]) -> BindGroupHandle{
        let render_pipeline = self.render_pipelines.get(pipeline).unwrap();
        let mut bind_group_entries = Vec::with_capacity(bindings.len());
        for binding in bindings {
            match binding.resource {
                BindingResource::Texture(id) => {
                    if let Some(texture) = self.textures.get(id) {
                        bind_group_entries.push(
                            wgpu::BindGroupEntry {
                                binding: binding.binding,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            }
                        )
                    }
                },
                BindingResource::Buffer(id) => {
                    if let Some(buffer) = self.buffers.get(id) {
                        bind_group_entries.push(
                            wgpu::BindGroupEntry {
                                binding: binding.binding,
                                resource: buffer.as_entire_binding(),
                            }
                        )
                    }
                },
            };
        }

        self.bind_groups.insert(
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.bind_group_layout,
                entries: &bind_group_entries[..],
            })
        )
    }

    pub fn new_compute_bind_group(&mut self, pipeline: ComputePipelineHandle, bindings: &[Binding]) -> BindGroupHandle{
        let render_pipeline = self.compute_pipelines.get(pipeline).unwrap();
        let mut bind_group_entries = Vec::with_capacity(bindings.len());
        for binding in bindings {
            match binding.resource {
                BindingResource::Texture(id) => {
                    if let Some(texture) = self.textures.get(id) {
                        bind_group_entries.push(
                            wgpu::BindGroupEntry {
                                binding: binding.binding,
                                resource: wgpu::BindingResource::TextureView(&texture.view),
                            }
                        )
                    }
                },
                BindingResource::Buffer(id) => {
                    if let Some(buffer) = self.buffers.get(id) {
                        bind_group_entries.push(
                            wgpu::BindGroupEntry {
                                binding: binding.binding,
                                resource: buffer.as_entire_binding(),
                            }
                        )
                    }
                },
            };
        }

        self.bind_groups.insert(
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &render_pipeline.bind_group_layout,
                entries: &bind_group_entries[..],
            })
        )
    }

    pub fn destroy_bind_group(&mut self, handle: BindGroupHandle) {
        self.bind_groups.remove(handle);
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.textures.insert(
            Texture { 
                texture,
                view
            }
        )
    }

    pub fn destroy_texture(&mut self, handle: TextureHandle) {
        self.textures.remove(handle);
    }

    pub fn update_texture(&mut self, handle: TextureHandle, desc: &wgpu::TextureDescriptor) {
        if let Some(texture) = self.textures.get_mut(handle) {
            texture.texture = self.device.create_texture(desc);
            texture.view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
        else {
            print!("Unknown texture")
        }
    }

    pub fn write_texture(
        &mut self, 
        handle: TextureHandle,
        data: &[u8],
        data_layout: ImageDataLayout,
        size: Extent3d
    ) {
        if let Some(texture) = self.textures.get_mut(handle) {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                }, 
                data, 
                data_layout, 
                size
            )
        }
        else {
            print!("Unknown texture")
        }
    }

    pub fn get_texture(&self, handle: TextureHandle) -> Option<&Texture> {
        self.textures.get(handle)
    }

    pub fn begin_frame(&self) -> Option<Frame> {
        match self.surface.get_current_texture() {
            Ok(surface_texture) => {
                let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });

                return Some(Frame::new(
                    &self,
                    surface_texture,
                    view,
                    encoder
                ));
            }
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => (), // self.resize(self.resolution),
            // The system is out of memory, we should probably quit
            //Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            // All other errors (Outdated, Timeout) should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }

        None
    }

    pub fn commit_frame(&self, frame: Frame) {    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        frame.surface_texture.present();
    }

    pub fn render_pipeline_count(&self) -> usize {
        self.render_pipelines.len()
    }

    pub fn compute_pipeline_count(&self) -> usize {
        self.compute_pipelines.len()
    }
    
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    pub fn bind_group_count(&self) -> usize {
        self.bind_groups.len()
    }

    pub fn shader_count(&self) -> usize {
        self.shaders.len()
    }

    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }
}