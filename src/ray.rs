use crate::vec3::{Point3, Vec3};
use num_traits::Float;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray<T> {
    pub origin: Point3<T>,
    pub direction: Vec3<T>,
}

impl<Scalar: Float> Ray<Scalar> {
    pub fn new(origin: Point3<Scalar>, direction: Vec3<Scalar>) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, time: Scalar) -> Vec3<Scalar> {
        self.origin + self.direction * time
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HitRecord<Scalar> {
    pub point: Point3<Scalar>,
    pub normal: Vec3<Scalar>,
    pub t: Scalar,
    pub is_front_face: bool,
}

impl<Scalar: Float> HitRecord<Scalar> {
    pub fn new(
        point: Point3<Scalar>,
        normal: Vec3<Scalar>,
        t: Scalar,
        is_front_face: bool,
    ) -> Self {
        HitRecord {
            point,
            normal,
            t,
            is_front_face,
        }
    }

    pub fn is_front_face(ray: &Ray<Scalar>, outward_normal: Vec3<Scalar>) -> bool {
        ray.direction.dot(outward_normal) < Scalar::zero()
    }

    /// `outward_normal` is assumed to be normalized already (unit length)
    pub fn face_normal(is_front_face: bool, outward_normal: Vec3<Scalar>) -> Vec3<Scalar> {
        if is_front_face {
            outward_normal
        } else {
            -outward_normal
        }
    }
}
