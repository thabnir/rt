use crate::{
    camera::Float,
    intersection::Intersection,
    material::Material,
    vec3::{Point3, Ray, RayExt, Vec3},
};
use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::{BHShape, BoundingHierarchy},
    bvh::Bvh,
};
use std::ops::Range;

pub struct World {
    pub shapes: Vec<Shape>,
    pub bvh: Bvh<Float, 3>,
}

impl World {
    /// Constructs a new `World` and builds its `BVH` in parallel
    pub fn build(mut shapes: Vec<Shape>) -> Self {
        let bvh = Bvh::build_par(&mut shapes);
        World { shapes, bvh }
    }
}

pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<Intersection>;
}

pub enum Shape {
    Sphere(Sphere),
}

impl Hit for Shape {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<Intersection> {
        match self {
            Shape::Sphere(s) => s.hit(ray, range),
        }
    }
}

impl Bounded<Float, 3> for Shape {
    fn aabb(&self) -> Aabb<Float, 3> {
        match self {
            Shape::Sphere(s) => s.aabb(),
        }
    }
}
impl BHShape<Float, 3> for Shape {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Shape::Sphere(s) => s.set_bh_node_index(index),
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            Shape::Sphere(s) => s.bh_node_index(),
        }
    }
}

impl Hit for World {
    /// Returns nearest hit to camera for the given ray within the given view range
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<Intersection> {
        // Save nearest collision to avoid checking for collisions against objects obscured by those we've already hit
        let mut nearest_hit_dist = range.end;
        let mut nearest_hit = None;

        // TODO: optimize this, don't need to test against every object for every ray
        // a BVH seems like the best option, though it's complicated
        // Also, matching an enum with all hittable types would likely improve performance vs OOP style
        // let hit_sphere_aabbs = bvh.traverse(&ray, &spheres);
        for shape in self.bvh.nearest_traverse_iterator(ray, &self.shapes) {
            if let Some(intersection) = shape.hit(ray, &(range.start..nearest_hit_dist)) {
                nearest_hit_dist = intersection.t;
                nearest_hit = Some(intersection);
            }
        }
        nearest_hit
    }
}

#[derive(Debug)]
pub struct Sphere {
    center: Point3,
    move_vec: Option<Vec3>,
    radius: Float,
    pub material: Material,
    /// For use in the BVH
    node_index: usize,
}

impl Bounded<f32, 3> for Sphere {
    fn aabb(&self) -> Aabb<f32, 3> {
        let half_size = Vec3::new(self.radius, self.radius, self.radius);
        let min = self.center - half_size;
        let max = self.center + half_size;
        Aabb::with_bounds(min.into(), max.into())
    }
}

impl BHShape<f32, 3> for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

impl Sphere {
    pub fn new(center: Vec3, radius: Float, material: Material) -> Self {
        Sphere {
            center,
            move_vec: None, // Stationary by default
            radius: radius.max(0.0),
            material,
            node_index: 0,
        }
    }

    pub fn new_moving(
        starting_center: Vec3,
        ending_center: Vec3,
        radius: Float,
        material: Material,
    ) -> Self {
        Sphere {
            center: starting_center,
            move_vec: Some(ending_center - starting_center),
            radius: radius.max(0.0),
            material,
            node_index: 0,
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
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<Intersection> {
        let oc = self.center - ray.origin.coords;
        let a = ray.direction.norm_squared();
        let h = ray.direction.dot(&oc);
        let c = oc.norm_squared() - self.radius * self.radius;

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
        let is_front_face = Intersection::is_front_face(ray, &normal);
        if !is_front_face {
            normal = -normal; // Set the normal to always face outward
        }

        Some(Intersection::new(
            point_on_sphere,
            normal,
            t,
            self.material,
            is_front_face,
        ))
    }
}
