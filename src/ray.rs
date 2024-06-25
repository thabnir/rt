use crate::vec3::{Point, Vec3};
use num_traits::Float;

pub struct Ray<Scalar: Float> {
    pub origin: Point<Scalar>,
    pub direction: Vec3<Scalar>,
}

impl<Scalar: Float> Ray<Scalar> {
    pub fn new(origin: Point<Scalar>, direction: Vec3<Scalar>) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, time: Scalar) -> Vec3<Scalar> {
        self.origin + self.direction * time
    }
}

pub struct HitRecord<Scalar: Float> {
    pub point: Point<Scalar>,
    pub normal: Vec3<Scalar>,
    pub t: Scalar,
    pub is_front_face: bool,
}

impl<Scalar: Float> HitRecord<Scalar> {
    // TODO:
    // desgin question: should this thing have any built-in support for partial initialization?
    // should someone be able to only calculate the normal and is_front_face when they decide to?
    // should it be somehow lazily done?
    // should i have `normal` and `is_front_face` be Optional types to help represent this?
    pub fn new(point: Point<Scalar>, normal: Vec3<Scalar>, t: Scalar, is_front_face: bool) -> Self {
        HitRecord {
            point,
            normal,
            t,
            is_front_face,
        }
    }

    /// `outward_normal` is assumed to be normalized already (unit length)
    /// TODO: remove this function?
    pub fn set_face_normal(&mut self, ray: &Ray<Scalar>, outward_normal: Vec3<Scalar>) {
        // TODO: figure out how to make this use approximate equality for floats in debug mode
        // https://github.com/brendanzab/approx/issues/24
        // debug_assert_abs_diff_eq!(outward_normal.length(), Scalar::one());
        self.is_front_face = Self::is_front_face(ray, outward_normal);
        self.normal = Self::face_normal(self.is_front_face, outward_normal);
    }

    pub fn is_front_face(ray: &Ray<Scalar>, outward_normal: Vec3<Scalar>) -> bool {
        ray.direction.dot(&outward_normal) < Scalar::zero()
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

pub trait Hittable<Scalar: Float> {
    fn hit(&self, ray: &Ray<Scalar>, t_min: Scalar, t_max: Scalar) -> Option<HitRecord<Scalar>>;
}

pub struct Sphere<Scalar: Float> {
    pub center: Point<Scalar>,
    pub radius: Scalar,
}

impl<Scalar: Float> Sphere<Scalar> {
    pub fn new(center: Point<Scalar>, radius: Scalar) -> Self {
        Sphere { center, radius }
    }
}

impl<Scalar: Float> Hittable<Scalar> for Sphere<Scalar> {
    fn hit(&self, ray: &Ray<Scalar>, t_min: Scalar, t_max: Scalar) -> Option<HitRecord<Scalar>> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(&oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < Scalar::zero() {
            return None; // no point hit on the sphere
        }
        let sqrt_disc = Float::sqrt(discriminant);
        // Find either root (hit point) in range
        let mut t = (h - sqrt_disc) / a; // min root
        if !(t_min..t_max).contains(&t) {
            t = (h + sqrt_disc) / a; // max root if min is out of range
            if !(t_min..t_max).contains(&t) {
                return None; // both out of range
            }
        }
        let point_on_sphere = ray.at(t);
        let normal = (point_on_sphere - self.center) / self.radius;
        let is_front_face = HitRecord::is_front_face(ray, normal);
        Some(HitRecord::new(point_on_sphere, normal, t, is_front_face))
    }
}
