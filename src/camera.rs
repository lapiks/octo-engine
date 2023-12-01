use crate::renderer_context::{BufferHandle, RendererContext};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    position: [f32; 3],
    focal_length: f32,
}

pub struct Camera {
    data: CameraData,
    buffer: BufferHandle,
}

impl Camera {
    pub fn new(renderer: &mut RendererContext, position: [f32; 3], focal_length: f32) -> Self {
        let data = CameraData {
            position,
            focal_length,
        };

        let buffer = renderer.new_buffer(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Globals buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        Camera { 
            data, 
            buffer,
        }
    }

    pub fn update_buffer(&mut self, renderer: &mut RendererContext) {
        renderer.update_buffer(self.buffer, bytemuck::bytes_of(&self.data))
    }
    
    pub fn binding_type(&self) -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub fn get_buffer(&self) -> BufferHandle {
        self.buffer
    }
} 
