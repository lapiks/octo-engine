use glam::UVec3;
use wgpu::Extent3d;

use crate::renderer_context::{TextureHandle, RendererContext};

pub struct VoxelWorld {
    data: [[[u8; 16]; 16]; 16],
    size: UVec3,
    texture: TextureHandle
}

impl VoxelWorld {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let data = [[[255; 16]; 16]; 16];

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
                format: wgpu::TextureFormat::R8Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        VoxelWorld { 
            data,
            size: UVec3::new(16, 16, 16),
            texture,
        }
    }

    pub fn set_voxel_at(&mut self, value: u8, coord: &UVec3) {
        self.data[coord.x as usize][coord.y as usize][coord.z as usize] = value; 
    }

    pub fn get_size(&self) -> &UVec3 {
        &self.size
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

    pub fn update_texture(&self, renderer: &mut RendererContext) {
        renderer.write_texture(
            self.texture, 
            bytemuck::bytes_of(&self.data), 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1 * self.size.x),
                rows_per_image: Some(self.size.y),
            }, 
            Extent3d {
                width: self.size.x,
                height: self.size.y,
                depth_or_array_layers: self.size.z,
            }
        );
    }
}