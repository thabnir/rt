use crate::{
    bvh::AxisAlignedBoundingBox,
    camera::Float,
    material::Material,
    ray::{HitRecord, Ray},
    vec3::{Vec3, Vec3Ext},
};
use rand::thread_rng;
use std::ops::{Deref, DerefMut, Range};

pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord>;
    /// Returns the bounding box over the entire range of motion
    fn bounding_box(&self) -> &AxisAlignedBoundingBox;
}

// TODO: figure out how to do this so it's basically a fancy wrapper for Vec except when i need to
// do stuff to it
// maybe something to with the adapter pattern? TBD
// add code here

pub struct World {
    hittable_list: Vec<Hittable>,
}

impl IntoIterator for World {
    type Item = Hittable;

    type IntoIter = <Vec<Hittable> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.hittable_list.into_iter()
    }
}

// TODO: decide whether I need these implementations of Deref and DerefMut
impl Deref for World {
    type Target = [Hittable];

    fn deref(&self) -> &Self::Target {
        &self.hittable_list[..]
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hittable_list[..]
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Adds the `hittable` to the world
    pub fn add(&mut self, hittable: Hittable) {
        self.hittable_list.push(hittable);
    }

    pub fn new() -> Self {
        World {
            hittable_list: Vec::new(),
        }
    }
}

pub enum Hittable {
    Sphere(Sphere),
}

impl Hit for World {
    /// Returns nearest hit to camera for the given ray within the given view range
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        // Save nearest collision to camera to avoid checking for collisions against objects obscured by those we've already hit
        let mut nearest_hit_dist = range.end;
        let mut nearest_hit = None;

        // TODO: optimize this, don't need to test against every object for every ray
        // a BVH seems like the best option, though it's complicated
        // Also, matching an enum with all hittable types would likely improve performance vs OOP style
        for hittable in self.iter() {
            match hittable {
                Hittable::Sphere(sphere) => {
                    if let Some(hit) = sphere.hit(ray, &(range.start..nearest_hit_dist)) {
                        nearest_hit_dist = hit.t;
                        nearest_hit = Some(hit);
                    }
                }
            }
        }

        nearest_hit
    }

    fn bounding_box(&self) -> &AxisAlignedBoundingBox {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct Sphere {
    center: Vec3,
    move_vec: Option<Vec3>,
    radius: Float,
    pub material: Material,
    bounding_box: AxisAlignedBoundingBox,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Float, material: Material) -> Self {
        let rvec = Vec3::new(radius, radius, radius);
        let bounding_box = AxisAlignedBoundingBox::new_from_points(center - rvec, center + rvec);
        Sphere {
            center,
            move_vec: None, // Stationary by default
            radius: radius.max(0.0),
            material,
            bounding_box,
        }
    }

    pub fn new_moving(
        starting_center: Vec3,
        ending_center: Vec3,
        radius: Float,
        material: Material,
    ) -> Self {
        let rvec = Vec3::new(radius, radius, radius);
        let aabb1 =
            AxisAlignedBoundingBox::new_from_points(starting_center - rvec, starting_center + rvec);
        let aabb2 =
            AxisAlignedBoundingBox::new_from_points(ending_center - rvec, ending_center + rvec);
        let bounding_box = AxisAlignedBoundingBox::around(aabb1, aabb2);
        Sphere {
            center: starting_center,
            move_vec: Some(ending_center - starting_center),
            radius: radius.max(0.0),
            material,
            bounding_box,
        }
    }

    pub fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
        let unit_vector: Vec3 = Vec3::random_unit(&mut thread_rng());
        if unit_vector.dot(*normal) > 0.0 {
            unit_vector // facing same direction as normal (out from sphere)
        } else {
            -unit_vector // facing toward center of sphere (must be inverted to reflect)
        }
    }

    pub fn center(&self, time: Float) -> Vec3 {
        if let Some(move_vec) = self.move_vec {
            self.center + move_vec * time // Lerp from starting to ending position
        } else {
            self.center
        }
    }
}

impl Hit for Sphere {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        let oc = self.center(ray.time) - ray.origin;
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
            self.material,
            is_front_face,
        ))
    }

    fn bounding_box(&self) -> &AxisAlignedBoundingBox {
        &self.bounding_box
    }
}
