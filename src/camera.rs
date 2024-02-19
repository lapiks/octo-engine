use glam::{Mat4, Vec2, Vec3};

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
    transform: Mat4,
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
            transform: Mat4::IDENTITY,
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
        self.transform.w_axis.x = pos.x;
        self.transform.w_axis.y = pos.y;
        self.transform.w_axis.z = pos.z;
    }

    pub fn translate(&mut self, v: Vec3) {        
        self.transform = Mat4::from_translation(v) * self.transform;
    }

    pub fn size(&self) -> Vec2 {
        Vec2::from_array(self.data.size)
    }

    pub fn update_buffer(&mut self, renderer: &mut RendererContext) {
        let (_, _, tr) = self.transform.to_scale_rotation_translation();
        self.data.position = tr.to_array();

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
