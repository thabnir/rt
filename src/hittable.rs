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
use enum_dispatch::enum_dispatch;
use std::{
    f64::consts::{PI, TAU},
    ops::Range,
};

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

#[enum_dispatch(Shape)]
pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<Intersection>;
}

#[enum_dispatch]
pub enum Shape {
    Sphere(Sphere),
}

// no fucking way this guy is literally me https://old.reddit.com/r/rust/comments/tgwpo7/avoiding_bad_patterns/
// he's even building a ray tracer + doing this because he heard it was faster
// TODO: kill the man who made the enum_dispatch library without support for supertraits
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
        // Only return the nearest collision
        let mut nearest_hit_dist = range.end;
        let mut nearest_hit = None;
        for shape in self.bvh.nearest_traverse_iterator(ray, &self.shapes) {
            if let Some(intersection) = shape.hit(ray, &(range.start..nearest_hit_dist)) {
                nearest_hit_dist = intersection.t;
                nearest_hit = Some(intersection);
            }
        }
        nearest_hit
    }
}

pub struct Sphere {
    center: Point3,
    radius: Float,
    /// To determine the rotation of the sphere (for textures)
    front_direction: Vec3,
    pub material: Material,
    /// For use in the BVH
    node_index: usize,
}

impl Sphere {
    /// Returns a new sphere facing in the direction `(1, 0, 0)`
    pub fn new(center: Vec3, radius: Float, material: Material) -> Self {
        Sphere {
            center,
            radius: radius.max(0.0),
            material,
            node_index: 0,
            front_direction: Vec3::x_axis().into_inner(),
        }
    }

    /// Returns a new sphere with its texture pointing in the direction of `front_face`
    pub fn new_facing(center: Vec3, radius: Float, material: Material, front_face: Vec3) -> Self {
        Sphere {
            center,
            radius: radius.max(0.0),
            material,
            node_index: 0,
            front_direction: front_face,
        }
    }
}

impl Bounded<Float, 3> for Sphere {
    fn aabb(&self) -> Aabb<Float, 3> {
        let half_size = Vec3::new(self.radius, self.radius, self.radius);
        let min = self.center - half_size;
        let max = self.center + half_size;
        Aabb::with_bounds(min.into(), max.into())
    }
}

impl BHShape<Float, 3> for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
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

        let (u, v) = unit_sphere_uv_facing(normal, self.front_direction);

        Some(Intersection::new(
            point_on_sphere,
            normal,
            t,
            &self.material,
            is_front_face,
            u,
            v,
        ))
    }
}

/// Returns the `(u, v)` coordinates of an `intersection_point` on the unit sphere centered at the
/// origin with the texture pitched, yawed, and rotated.
/// Uses **radians**
pub fn unit_sphere_uv(
    intersection_point: Point3,
    pitch_rads: Float,
    yaw_rads: Float,
    rotation_rads: Float,
) -> (Float, Float) {
    let rotation_matrix = nalgebra::Rotation3::from_euler_angles(0.0, pitch_rads, 0.0)
        * nalgebra::Rotation3::from_euler_angles(0.0, 0.0, -yaw_rads);

    let rotated_point = rotation_matrix * intersection_point;
    let (theta, phi) = to_unit_spherical(rotated_point);

    let phi = (phi + rotation_rads).rem_euclid(TAU); // Rotates the texture around its pole

    let u = phi / TAU;
    let v = theta / PI;

    (u, v)
}

/// Returns the `(u, v)` coordinates of an `intersection_point` on the unit sphere centered at the
/// origin with the texture facing toward `face_dir`
fn unit_sphere_uv_facing(intersection_point: Point3, face_dir: Vec3) -> (Float, Float) {
    let pitch = face_dir
        .z
        .atan2((face_dir.y * face_dir.y + face_dir.x * face_dir.x).sqrt());
    let yaw = face_dir.y.atan2(face_dir.x);

    let rotation = 0.0;
    unit_sphere_uv(intersection_point, pitch, yaw, rotation)
}

fn to_unit_spherical(point: Point3) -> (Float, Float) {
    let theta = (-point.z).acos();
    let phi = Float::atan2(point.y, point.x) + PI;
    (theta, phi)
}
