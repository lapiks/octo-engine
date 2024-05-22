use crate::{renderer_context::{BindGroupHandle, Binding, BindingResource, ComputePipelineHandle, PipelineDesc, RendererContext, ShaderHandle}, voxel_world::VoxelWorld};


pub struct Sprite {
    pub compute_shader: Option<ShaderHandle>,
    pub compute_pipeline: ComputePipelineHandle,
    pub compute_bind_group: BindGroupHandle,
}

impl Sprite {
    pub fn new(renderer: &mut RendererContext, world: &VoxelWorld) -> Self {
        let shader_src = std::fs::read_to_string("src/shaders/compute_cube.wgsl").unwrap();
        let compute_shader = 
            match renderer.new_shader(shader_src.as_str()) {
                Ok(shader) => Some(shader),
                Err(e) => 
                {
                    println!("error: {}", e);
                    None
                }
            };

        let compute_pipeline = renderer.new_compute_pipeline(
            &PipelineDesc {
                shader: compute_shader.unwrap(),
                bindings_layout: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::R32Uint,
                            view_dimension: wgpu::TextureViewDimension::D3,
                        },
                        count: None,
                    },
                ]
            }
        );

        let compute_bind_group = renderer.new_compute_bind_group(
            compute_pipeline, 
            &[
            Binding {
                binding: 0,
                resource: BindingResource::Texture(world.get_texture()),
            }]
        );

        Self {
            compute_shader,
            compute_pipeline,
            compute_bind_group,
        }
    }
}