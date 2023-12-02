use crate::{renderer_context::{RendererContext, BufferHandle}, buffer_resource::BufferResource};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalsData {
    pub width: u32,
    pub height: u32,
}

pub struct Globals {
    data: GlobalsData,
    buffer: BufferResource,
}

impl Globals {
    pub fn new(renderer: &mut RendererContext, width: u32, height: u32) -> Self {
        let data = GlobalsData {
            width,
            height,
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
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    }

    pub fn get_buffer(&self) -> BufferHandle {
        self.buffer.get_buffer()
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.data.width = width;
        self.data.height = height;
    }
} 
