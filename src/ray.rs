use glam::{Vec3, Mat4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn transform(&self, mat: &Mat4) -> Ray {
        Ray {
            origin:  mat.transform_point3(self.origin),
            direction: mat.transform_vector3(self.direction)
        }
    }
}