use bytemuck::{Pod, Zeroable};

use crate::renderer_context::{BufferHandle, RendererContext};

pub struct BufferResource {
    buffer: BufferHandle,
}

impl BufferResource {
    pub fn new<T>(renderer: &mut RendererContext, data: &T) -> Self
    where T: Pod + Zeroable + Clone + Copy {
        let buffer = renderer.new_buffer(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Globals buffer"),
                contents: bytemuck::bytes_of(data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        BufferResource {
            buffer,
        }
    }

    pub fn update_buffer<T>(&mut self, renderer: &mut RendererContext, data: &T) 
    where T: Pod + Zeroable + Clone + Copy {
        renderer.update_buffer(self.buffer, bytemuck::bytes_of(data))
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