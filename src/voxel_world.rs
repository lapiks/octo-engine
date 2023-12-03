use crate::renderer_context::{TextureHandle, RendererContext};

struct VoxelWorldData {

}

pub struct VoxelWorld {
    data: VoxelWorldData,  
    texture: TextureHandle
}

impl VoxelWorld {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let data = VoxelWorldData {};

        let texture = renderer.new_texture(
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: 16,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba8Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        VoxelWorld { 
            data,
            texture,
        }
    }

    pub fn get_texture(&self) -> TextureHandle {
        self.texture
    }

    pub fn binding_type() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Uint,
            view_dimension: wgpu::TextureViewDimension::D3,
            multisampled: false,
        }
    }
}