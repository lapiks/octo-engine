use slotmap::{SlotMap, new_key_type};

mod renderer_context;

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

