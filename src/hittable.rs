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
use nalgebra::Matrix4;
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use std::{
    f64::consts::{PI, TAU},
    ops::Range,
    sync::Arc,
};
use tobj::GPU_LOAD_OPTIONS;

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
    Sphere,
    Triangle,
}

// no fucking way this guy is literally me https://old.reddit.com/r/rust/comments/tgwpo7/avoiding_bad_patterns/
// TODO: kill the man who made the enum_dispatch library without support for supertraits
impl Bounded<Float, 3> for Shape {
    fn aabb(&self) -> Aabb<Float, 3> {
        match self {
            Shape::Sphere(s) => s.aabb(),
            Shape::Triangle(t) => t.aabb(),
        }
    }
}

impl BHShape<Float, 3> for Shape {
    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Shape::Sphere(s) => s.set_bh_node_index(index),
            Shape::Triangle(t) => t.set_bh_node_index(index),
        }
    }

    fn bh_node_index(&self) -> usize {
        match self {
            Shape::Sphere(s) => s.bh_node_index(),
            Shape::Triangle(t) => t.bh_node_index(),
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

// TODO: look up best design practices for triangles in a ray tracer
pub struct Triangle {
    pub a: Point3,
    pub b: Point3,
    pub c: Point3,
    normal: Vec3,
    pub material: Arc<Material>,
    node_index: usize,
}

impl Triangle {
    pub fn new(a: Point3, b: Point3, c: Point3, material: Arc<Material>) -> Self {
        let ac = c - a;
        let bc = c - b;
        // relies on the order of a, b, c in the definition for the "front face" direction
        // TODO: the normal for the triangle is NOT correct
        Triangle {
            a,
            b,
            c,
            normal: ac.cross(&bc), // TODO: does this make sense?
            material,
            node_index: 0,
        }
    }

    pub fn transform(&self, matrix: &Matrix4<Float>) -> Self {
        let a = matrix.transform_vector(&self.a);
        let b = matrix.transform_vector(&self.b);
        let c = matrix.transform_vector(&self.c);
        Triangle::new(a, b, c, self.material.clone())
    }

    pub fn shift(&self, shift: Vec3) -> Self {
        Triangle::new(
            self.a + shift,
            self.b + shift,
            self.c + shift,
            self.material.clone(),
        )
    }
}

impl Bounded<Float, 3> for Triangle {
    fn aabb(&self) -> Aabb<Float, 3> {
        let min = self.a.inf(&self.b).inf(&self.c);
        let max = self.a.sup(&self.b).sup(&self.c);
        Aabb::with_bounds(min.into(), max.into())
    }
}

impl BHShape<Float, 3> for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

pub struct Sphere {
    center: Point3,
    radius: Float,
    /// To determine the rotation of the sphere (for textures)
    front_direction: Vec3,
    pub material: Arc<Material>,
    /// For use in the BVH
    node_index: usize,
}

impl Sphere {
    /// Returns a new sphere facing in the direction of the x-axis
    pub fn new(center: Vec3, radius: Float, material: Arc<Material>) -> Self {
        Sphere {
            center,
            radius: radius.max(0.0),
            material,
            node_index: 0,
            front_direction: Vec3::x_axis().into_inner(),
        }
    }

    /// Returns a new sphere with its texture pointing in the direction of `front_face`
    pub fn new_facing(
        center: Vec3,
        radius: Float,
        material: Arc<Material>,
        front_face: Vec3,
    ) -> Self {
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
            return None; // no point hit on the sphere OR glancing hit
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
            // return None;
            normal = -normal; // Set the normal to always face outward
        }

        let (u, v) = unit_sphere_uv_facing(normal, self.front_direction);

        if u.is_nan() || v.is_nan() {
            // TODO: figure out how to avoid this
            // println!("bang");
            return None; // NaN occurs sometimes with glancing blows on the sphere
        }

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
    let theta = (-point.z).acos(); // acos is slow as balls
    let phi = Float::atan2(point.y, point.x) + PI;
    (theta, phi)
}

impl Hit for Triangle {
    // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    // This is adapted from `intersects_triangle` in the BVH crate
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<Intersection> {
        let a_to_b = self.b - self.a;
        let a_to_c = self.c - self.a;

        // Begin calculating determinant - also used to calculate u parameter
        // u_vec lies in view plane
        // length of a_to_c in view_plane = |u_vec| = |a_to_c|*sin(a_to_c, dir)
        let u_vec = ray.direction.cross(&a_to_c);

        // If determinant is near zero, ray lies in plane of triangle
        // The determinant corresponds to the parallelepiped volume:
        // det = 0 => [dir, a_to_b, a_to_c] not linearly independant
        let det = a_to_b.dot(&u_vec);

        // Only testing positive bound, thus enabling backface culling
        // If backface culling is not desired write:
        // det < EPSILON && det > -EPSILON
        if det < Float::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        // Vector from point a to ray origin
        let a_to_origin = ray.origin - self.a;

        // Calculate u parameter
        let u = a_to_origin.coords.dot(&u_vec) * inv_det;

        // Test bounds: u < 0 || u > 1 => outside of triangle
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        // Prepare to test v parameter
        let v_vec = a_to_origin.coords.cross(&a_to_b);

        // Calculate v parameter and test bound
        let v = ray.direction.dot(&v_vec) * inv_det;
        // The intersection lies outside of the triangle
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let dist = a_to_c.dot(&v_vec) * inv_det;
        if !range.contains(&dist) {
            return None;
        }

        if dist > Float::EPSILON {
            // TODO: verify this all. Much is handwaved and halfassed and untested
            let intersection_point = ray.origin.coords + ray.direction * dist;
            let is_front_face = ray.direction.dot(&self.normal) <= 0.0;
            Some(Intersection::new(
                intersection_point,
                self.normal,
                dist,
                &self.material,
                is_front_face,
                u,
                v,
            ))
        } else {
            None
        }
    }
}

pub fn load_obj(
    file_path: &str,
    mesh_material: Arc<Material>,
    transform: Option<Matrix4<Float>>,
    centered: bool,
) -> Vec<Vec<Triangle>> {
    let options = GPU_LOAD_OPTIONS;

    let (models, _materials) =
        tobj::load_obj(file_path, &options).expect("Failed to OBJ load file");

    let mut models_triangled = Vec::new();

    for model in models {
        let positions: Vec<Point3> = model
            .mesh
            .positions
            .par_chunks_exact(3)
            .map(|v| Point3::new(Float::from(v[0]), Float::from(v[1]), Float::from(v[2])))
            .collect();

        let default = Matrix4::identity();

        let mut sum_pos = Vec3::zeros();
        let triangles: Vec<Triangle> = model
            .mesh
            .indices
            .chunks_exact(3)
            .map(|idx| {
                let a = positions[idx[0] as usize];
                let b = positions[idx[1] as usize];
                let c = positions[idx[2] as usize];

                sum_pos += a + b + c;

                Triangle::new(a, b, c, mesh_material.clone())
                    .transform(&transform.unwrap_or(default))
            })
            .collect();

        // TODO: centering doesn't work at all for some reason
        if centered {
            let mean_pos = sum_pos / (model.mesh.positions.len() / 3) as Float;
            println!("mean_pos: {}", mean_pos);
            let centered_tris: Vec<Triangle> =
                triangles.iter().map(|tri| tri.shift(-mean_pos)).collect();
            let new_center = centered_tris
                .iter()
                .fold(Vec3::zeros(), |sum, v| sum + v.a + v.b + v.c);
            println!("new center: {} (not divided)", new_center);
            models_triangled.push(centered_tris);
        } else {
            models_triangled.push(triangles);
        }
    }

    models_triangled
}
