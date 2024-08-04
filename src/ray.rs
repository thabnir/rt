use crate::{camera::Float, material::Material, vec3::Vec3};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub time: Float,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, time: Float) -> Self {
        Self {
            origin,
            direction,
            time,
        }
    }

    pub fn at(&self, time: Float) -> Vec3 {
        self.origin + self.direction * time
    }
}

#[derive(Clone)]
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub t: Float,
    pub is_front_face: bool,
}

impl HitRecord {
    pub fn new(
        point: Vec3,
        normal: Vec3,
        t: Float,
        material: Material,
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
