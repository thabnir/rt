use crate::{camera::Float, material::Scatter};
use glam::Vec3;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, time: Float) -> Vec3 {
        self.origin + self.direction * time
    }
}

#[derive(Clone)]
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Arc<dyn Scatter + Send + Sync>,
    pub t: Float,
    pub is_front_face: bool,
}

impl HitRecord {
    pub fn new(
        point: Vec3,
        normal: Vec3,
        t: Float,
        material: Arc<dyn Scatter + Send + Sync>,
        is_front_face: bool,
    ) -> Self {
        HitRecord {
            point,
            normal,
            material,
            t,
            is_front_face,
        }
    }

    pub fn is_front_face(ray: &Ray, outward_normal: Vec3) -> bool {
        ray.direction.dot(outward_normal) < 0.0
    }
}
