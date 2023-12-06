use glam::Vec2;

use crate::{renderer_context::{RendererContext, BufferHandle}, buffer_resource::BufferResource};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalsData {
    pub screen_size: [f32; 2],
}

pub struct Globals {
    data: GlobalsData,
    buffer: BufferResource,
}

impl Globals {
    pub fn new(renderer: &mut RendererContext, screen_size: Vec2) -> Self {
        let data = GlobalsData {
            screen_size: screen_size.to_array()
        };

        let buffer = BufferResource::new(renderer, &data);

        Globals {
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

    pub fn set_size(&mut self, screen_size: Vec2) {
        self.data.screen_size = screen_size.to_array();
    }

    pub fn get_size(&self) -> Vec2 {
        Vec2::from_array(self.data.screen_size)
    }
} 
