use crate::{renderer_context::{RendererContext, BufferHandle}, buffer_resource::BufferResource};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    position: [f32; 3],
    focal_length: f32,
}

pub struct Camera {
    data: CameraData,
    buffer: BufferResource,
}

impl Camera {
    pub fn new(renderer: &mut RendererContext, position: [f32; 3], focal_length: f32) -> Self {
        let data = CameraData {
            position,
            focal_length,
        };

        let buffer = BufferResource::new(renderer, &data);

        Camera { 
            data, 
            buffer,
        }
    }

    pub fn update_buffer(&mut self, renderer: &mut RendererContext) {
        self.buffer.update_buffer(renderer, &self.data);
    }
    
    pub fn binding_type(&self) -> wgpu::BindingType {
        return self.buffer.binding_type()
    }

    pub fn get_buffer(&self) -> BufferHandle {
        self.buffer.get_buffer()
    }
} 
