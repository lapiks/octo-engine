use crate::{renderer_context::{RendererContext, BufferHandle}, buffer_resource::BufferResource};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalsData {}

pub struct Globals {
    data: GlobalsData,
    buffer: BufferResource,
}

impl Globals {
    pub fn new(renderer: &mut RendererContext) -> Self {
        let data = GlobalsData {};

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
} 
