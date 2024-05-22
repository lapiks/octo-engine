use glam::UVec3;
use wgpu::Extent3d;

use crate::renderer_context::{TextureHandle, RendererContext};

const WORLD_SIZE: usize = 32; 

pub struct VoxelWorld {
    data: [[[u32; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE],
    size: UVec3,
    texture: TextureHandle
}

impl VoxelWorld {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let data = [[[0; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE];

        let texture = renderer.new_texture(
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: WORLD_SIZE as u32,
                    height: WORLD_SIZE as u32,
                    depth_or_array_layers: WORLD_SIZE as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: wgpu::TextureFormat::R32Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            }
        );

        VoxelWorld { 
            data,
            size: UVec3::new(WORLD_SIZE as u32, WORLD_SIZE as u32, WORLD_SIZE as u32),
            texture,
        }
    }

    pub fn set_voxel_at(&mut self, value: u32, coord: &UVec3) {
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
                bytes_per_row: Some(4 * self.size.x),
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