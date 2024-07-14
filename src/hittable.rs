use crate::{
    camera::Float,
    material::Scatter,
    ray::{HitRecord, Ray},
    vec3_ext::Vec3Ext,
};
use glam::Vec3;
use rand::thread_rng;
use std::{ops::Range, sync::Arc};

pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord>;
}

pub type World = Vec<Box<dyn Hit>>;

impl Hit for World {
    /// Returns nearest hit to camera for the given ray within the given view range
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        // Save nearest collision to camera to avoid checking for collisions against objects obscured by those we've already hit
        let mut nearest_hit_dist = range.end;
        let mut nearest_hit = None;

        // TODO: optimize this, don't need to test against every object for every ray
        // a BVH seems like the best option, though it's complicated
        for obj in self.iter() {
            if let Some(hit) = obj.hit(ray, &(range.start..nearest_hit_dist)) {
                nearest_hit_dist = hit.t;
                nearest_hit = Some(hit);
            }
        }

        nearest_hit
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: Float,
    pub material: Arc<dyn Scatter + Send + Sync>,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Float, material: impl Scatter + 'static) -> Self {
        let radius = radius.max(0.0);
        Sphere {
            center,
            radius,
            material: Arc::new(material),
        }
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
        let unit_vector: Vec3 = Vec3::random_unit(&mut thread_rng());
        if unit_vector.dot(*normal) > 0.0 {
            return unit_vector; // facing same direction as normal (out from sphere)
        }
        -unit_vector // facing toward center of sphere (must be inverted to reflect)
    }
}

impl Hit for Sphere {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None; // no point hit on the sphere
        }

        let sqrt_disc = discriminant.sqrt();
        // Find either root (hit point) in range
        let mut t = (h - sqrt_disc) / a; // min root
        if !(range).contains(&t) {
            t = (h + sqrt_disc) / a; // max root if min is out of range
            if !(range).contains(&t) {
                return None; // both out of range
            }
        }

        let point_on_sphere = ray.at(t);
        let mut normal = (point_on_sphere - self.center) / self.radius;
        let is_front_face = HitRecord::is_front_face(ray, normal);
        if !is_front_face {
            normal = -normal; // Set the normal to always face outward
        }

        Some(HitRecord::new(
            point_on_sphere,
            normal,
            t,
            self.material.clone(), // clones the Arc, not the material
            is_front_face,
        ))
    }
}
