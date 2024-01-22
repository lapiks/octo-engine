use glam::{Vec3, Mat4, Vec2, vec3};

use crate::{renderer_context::{RendererContext, BufferHandle}, buffer_resource::BufferResource, ray::Ray, color::Color};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraData {
    position: [f32; 3],
    _padding: u32,
    size: [f32; 2],
    focal_length: f32,
    _padding2: u32,
}

pub struct Camera {
    transform: Mat4,
    transform_inverse: Mat4,
    pixel_size: f32,
    half_width: f32,
    half_height: f32,
    fov: f32,
    background: Color,
    data: CameraData,
    buffer: BufferResource,
}

impl Camera {
    pub fn new(renderer: &mut RendererContext, position: Vec3, size: Vec2, focal_length: f32, fov: f32) -> Self {
        let data = CameraData {
            position: position.to_array(),
            _padding: 0,
            size: size.to_array(),
            focal_length,
            _padding2: 0,
        };

        let buffer = BufferResource::new(renderer, &data);

        let mut camera = Camera { 
            transform: Mat4::IDENTITY,
            transform_inverse: Mat4::IDENTITY,
            pixel_size: 0.0,
            half_width: 0.0,
            half_height: 0.0,
            fov,
            background: Color::white(),
            data, 
            buffer,
        };
        camera.set_size(size);
        camera
    }

    pub fn set_size(&mut self, size: Vec2) {
        let aspect = size.x / size.y;
        let half_view = f32::tan(self.fov / 2.0);
        self.half_width = 
            if aspect >= 1.0 { 
                half_view 
            } else { 
                half_view * aspect 
            };
        self.half_height = 
            if aspect >= 1.0 { 
                half_view / aspect
            } else { 
                half_view 
            };
        self.pixel_size = (self.half_width * 2.0) / size.x;

        self.data.size = size.to_array();
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.data.position = pos.to_array();
    }

    pub fn translate(&mut self, v: Vec3) {
        self.data.position[0] += v.x;
        self.data.position[1] += v.y;
        self.data.position[2] += v.z;
    }

    pub fn size(&self) -> Vec2 {
        Vec2::from_array(self.data.size)
    }

    pub fn ray_for_pixel(&self, x: f32, y: f32) -> Ray {
        let world_x = self.half_width - (x + 0.5) * self.pixel_size;
        let world_y = self.half_height - (y + 0.5) * self.pixel_size;
        let pixel = self.transform_inverse.transform_point3(vec3(world_x, world_y, -1.0));
        let origin = self.transform_inverse.transform_point3(Vec3::ZERO);
        let direction = (pixel - origin).normalize();
        Ray {
            origin,
            direction 
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
