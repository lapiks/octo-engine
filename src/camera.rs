use glam::{Vec3, Vec2};

use crate::renderer_context::{RendererContext, BufferHandle};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    position: [f32; 3],
    _padding: u32,
    direction: [f32; 3],
    _padding2: u32,
    size: [f32; 2],
    _padding3: u64,
}

pub struct Camera {
    data: CameraData,
    buffer: BufferHandle,
}

impl Camera {
    pub fn new(renderer: &mut RendererContext, size: Vec2) -> Self {
        let data = CameraData {
            position: [0.0, 0.0, 0.0],
            _padding: 0,
            direction: [0.0, 0.0, 1.0],
            _padding2: 0,
            size: size.to_array(),
            _padding3: 0,
        };

        let buffer = renderer.new_buffer(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Resource buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let mut camera = Camera { 
            data, 
            buffer,
        };
        camera.set_size(size);
        camera
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.data.size = size.to_array();
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.data.position = pos.to_array();
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        self.data.direction = dir.to_array();
    }

    pub fn translate(&mut self, v: Vec3) {
        self.data.position[0] += v.x;
        self.data.position[1] += v.y;
        self.data.position[2] += v.z;
    }

    pub fn size(&self) -> Vec2 {
        Vec2::from_array(self.data.size)
    }

    pub fn update_buffer(&mut self, renderer: &mut RendererContext) {
        renderer.update_buffer(self.buffer, bytemuck::bytes_of(&self.data));
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
