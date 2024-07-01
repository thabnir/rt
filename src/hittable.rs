use crate::{ray::HitRecord, vec3::Vec3};
use std::ops::Range;

use num_traits::Float;
use rand::{distributions::uniform::SampleUniform, thread_rng};

use crate::{ray::Ray, vec3::Point3};

pub trait Hittable<Scalar: Float>: Send + Sync {
    fn hit(&self, ray: &Ray<Scalar>, range: &Range<Scalar>) -> Option<HitRecord<Scalar>>;
}

pub type World<Scalar> = Vec<Box<dyn Hittable<Scalar>>>;

impl<Scalar: Float + Send + Sync> Hittable<Scalar> for World<Scalar> {
    /// Returns nearest hit to camera for the given ray within the given view range
    fn hit(&self, ray: &Ray<Scalar>, range: &Range<Scalar>) -> Option<HitRecord<Scalar>> {
        // Save nearest collision to camera to avoid checking for collisions against objects obscured by those we've already hit
        let mut nearest_hit_dist = range.end;
        let mut nearest_hit = None;

        for obj in self.iter() {
            if let Some(hit) = obj.hit(ray, &(range.start..nearest_hit_dist)) {
                nearest_hit_dist = hit.t;
                nearest_hit = Some(hit);
            }
        }

        nearest_hit
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere<Scalar> {
    pub center: Point3<Scalar>,
    pub radius: Scalar,
}

impl<T> Sphere<T> {
    pub fn new(center: Point3<T>, radius: T) -> Self {
        Sphere { center, radius }
    }
}

impl<T: Float + SampleUniform> Sphere<T> {
    pub fn random_on_hemisphere(normal: &Vec3<T>) -> Vec3<T> {
        let unit_vector: Vec3<T> = Vec3::random_unit(&mut thread_rng());
        if unit_vector.dot(*normal) > T::zero() {
            return unit_vector; // facing same direction as normal (out from sphere)
        }
        -unit_vector // facing toward center of sphere (must be inverted to reflect)
    }
}

impl<Scalar: Float + Send + Sync> Hittable<Scalar> for Sphere<Scalar> {
    fn hit(&self, ray: &Ray<Scalar>, range: &Range<Scalar>) -> Option<HitRecord<Scalar>> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < Scalar::zero() {
            return None; // no point hit on the sphere
        }
        let sqrt_disc = Float::sqrt(discriminant);
        // Find either root (hit point) in range
        let mut t = (h - sqrt_disc) / a; // min root
        if !(range).contains(&t) {
            t = (h + sqrt_disc) / a; // max root if min is out of range
            if !(range).contains(&t) {
                return None; // both out of range
            }
        }
        let point_on_sphere = ray.at(t);
        let normal = (point_on_sphere - self.center) / self.radius;
        let is_front_face = HitRecord::is_front_face(ray, normal);
        Some(HitRecord::new(point_on_sphere, normal, t, is_front_face))
    }
}