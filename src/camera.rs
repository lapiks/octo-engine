use glam::{Mat4, Vec2, Vec3};

use crate::{renderer_context::{BufferHandle, RendererContext}, transform::Transform};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    model: [[f32; 4]; 4],
    size: [f32; 2],
    _padding3: u64,
}

pub struct Camera {
    pub transform: Transform,
    data: CameraData,
    buffer: BufferHandle,
}

impl Camera {
    pub fn new(renderer: &mut RendererContext, size: Vec2) -> Self {
        let data = CameraData {
            model: Mat4::IDENTITY.to_cols_array_2d(),
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
            transform: Transform::IDENTITY,
            data, 
            buffer,
        };
        camera.set_size(size);
        camera
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.data.size = size.to_array();
    }

    pub fn size(&self) -> Vec2 {
        Vec2::from_array(self.data.size)
    }

    pub fn update_buffer(&mut self, renderer: &mut RendererContext) {
        self.data.model = self.transform.compute_matrix().to_cols_array_2d();

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
