use crate::{
    camera::Float,
    intersection::Intersection,
    material::Material,
    vec3::{Point3, Ray, RayExt, Vec2, Vec3, Vec3Ext},
};
use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::{BHShape, BoundingHierarchy},
    bvh::Bvh,
};
use enum_dispatch::enum_dispatch;
use hw_skymodel::rgb::{Channel, SkyParams, SkyState};
use nalgebra::Matrix4;
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use std::{
    f64::consts::{PI, TAU},
    ops::Range,
    sync::Arc,
};
use tobj::GPU_LOAD_OPTIONS;

// TODO: make shapes and bvh private and turn their usage into an iterator
pub struct World {
    pub shapes: Vec<Shape>,
    pub bvh: Bvh<Float, 3>,
    sky: SkyState,
    sun_direction: Vec3,
}

impl World {
    /// Constructs a new `World` and builds its `BVH` in parallel
    pub fn build(mut shapes: Vec<Shape>) -> Self {
        let bvh = Bvh::build_par(&mut shapes);
        let sky = SkyState::new(&SkyParams::default()).expect("error constructing sky model");

        // TODO: test best default sun direction, maybe add parameter in `build`
        let sun_direction = Vec3::new(0.0, 0.0, 1.0).normalize();

        World {
            shapes,
            bvh,
            sky,
            sun_direction,
        }
    }

    // Taken from this blog post: https://nelari.us/post/weekend_raytracing_with_wgpu_2/
    // Notes on tomemapping and color space transformations: https://computergraphics.stackexchange.com/questions/10315/tone-mapping-vs-gamma-correction
    // In essence: yes, keep the gamma correction at the end.
    fn uncharted2_tonemap(x: Vec3) -> Vec3 {
        let a = 0.15;
        let b = 0.50;
        let c = 0.10;
        let d = 0.20;
        let e = 0.02;
        let f = 0.30;
        // let w = 11.2;

        let numerator = x.component_mul(&(a * x + Vec3::new(c * b, c * b, c * b)))
            + Vec3::new(d * e, d * e, d * e);
        let denominator =
            x.component_mul(&(a * x + Vec3::new(b, b, b))) + Vec3::new(d * f, d * f, d * f);

        numerator.component_div(&denominator) - Vec3::new(e / f, e / f, e / f)
    }

    /// Takes an `unclamped_color` and returns a color with values in the range [0.0, 1.0]
    /// [Taken from this blog post](https://nelari.us/post/weekend_raytracing_with_wgpu_2/)
    fn uncharted2(x: Vec3) -> Vec3 {
        // let exposure_bias = 0.246; // determined experimentally for the scene
        let exposure_bias = 1.1;

        let curr = World::uncharted2_tonemap(exposure_bias * x);

        let w = 11.2;
        let white_scale = Vec3::ONE.component_div(&World::uncharted2_tonemap(Vec3::new(w, w, w)));
        white_scale.component_mul(&curr)
    }

    // TODO: stop clamping any colors before the final display in the window
    // only tonemap them right before. that way shit can have greater contrast and emit light
    // wait is that even true? hmmmmmmmmmmmmmmmmmmmmmmmmmm
    pub fn sky_color_toward(&self, direction: &Vec3) -> Vec3 {
        let theta = direction.z.acos() as f32;
        let gamma = direction.dot(&self.sun_direction).clamp(-1.0, 1.0) as f32;
        let color = Vec3::new(
            self.sky.radiance(theta, gamma, Channel::R).into(),
            self.sky.radiance(theta, gamma, Channel::G).into(),
            self.sky.radiance(theta, gamma, Channel::B).into(),
        );
        World::uncharted2(color)
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
// he's even building a ray tracer + doing this because he heard it was faster
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
#[derive(Debug)]
pub struct Triangle {
    pub a: Point3,
    pub b: Point3,
    pub c: Point3,
    pub uv_a: Vec2,
    pub uv_b: Vec2,
    pub uv_c: Vec2,
    normal: Vec3,
    pub material: Arc<Material>,
    node_index: usize,
}

impl Triangle {
    pub fn new(a: Point3, b: Point3, c: Point3, material: Arc<Material>) -> Self {
        // Normalizing early and often to avoid numerical errors
        // Shouldn't matter for performance since shapes are only created once
        let ab = (b - a).normalize();
        let ac = (c - a).normalize();
        Triangle {
            a,
            b,
            c,
            uv_a: Vec2::new(0.0, 0.0), // 0.0, 0.0
            uv_b: Vec2::new(1.0, 0.0), // 1.0, 0.0
            uv_c: Vec2::new(0.5, 1.0), // 0.5, 1.0
            normal: ab.cross(&ac).normalize(),
            material,
            node_index: 0,
        }
    }

    pub fn new_with_uv(
        a: Point3,
        b: Point3,
        c: Point3,
        uv_a: Vec2,
        uv_b: Vec2,
        uv_c: Vec2,
        material: Arc<Material>,
    ) -> Self {
        // Normalizing early and often to avoid numerical errors
        // Shouldn't matter for performance since shapes are only created once
        let ab = (b - a).normalize();
        let ac = (c - a).normalize();
        Triangle {
            a,
            b,
            c,
            uv_a,
            uv_b,
            uv_c,
            normal: ab.cross(&ac).normalize(),
            material,
            node_index: 0,
        }
    }

    pub fn new_opposite_normal(a: Point3, b: Point3, c: Point3, material: Arc<Material>) -> Self {
        Self::new(c, b, a, material)
    }

    pub fn transform(&self, matrix: &Matrix4<Float>) -> Self {
        let a = matrix.transform_vector(&self.a);
        let b = matrix.transform_vector(&self.b);
        let c = matrix.transform_vector(&self.c);
        Triangle::new_with_uv(
            a,
            b,
            c,
            self.uv_a,
            self.uv_b,
            self.uv_c,
            self.material.clone(),
        )
    }

    pub fn shift(&self, shift: Vec3) -> Self {
        Triangle::new_with_uv(
            self.a + shift,
            self.b + shift,
            self.c + shift,
            self.uv_a,
            self.uv_b,
            self.uv_c,
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

        let uv = unit_sphere_uv_facing(normal, self.front_direction);

        if uv.x.is_nan() || uv.y.is_nan() {
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
            uv,
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
) -> Vec2 {
    let rotation_matrix = nalgebra::Rotation3::from_euler_angles(0.0, pitch_rads, 0.0)
        * nalgebra::Rotation3::from_euler_angles(0.0, 0.0, -yaw_rads);

    let rotated_point = rotation_matrix * intersection_point;
    let (theta, phi) = to_unit_spherical(rotated_point);

    let phi = (phi + rotation_rads).rem_euclid(TAU); // Rotates the texture around its pole

    let u = phi / TAU;
    let v = theta / PI;

    Vec2::new(u, v)
}

/// Returns the `(u, v)` coordinates of an `intersection_point` on the unit sphere centered at the
/// origin with the texture facing toward `face_dir`
fn unit_sphere_uv_facing(intersection_point: Point3, face_dir: Vec3) -> Vec2 {
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
            // TODO: add flag for backface culling on triangles
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

            // Interpolate the UV coordinates at the hit point
            // let uv_no_map = Vec2::new(u, v);

            let left = self.uv_a.x.min(self.uv_b.x).min(self.uv_c.x);
            let right = self.uv_a.x.max(self.uv_b.x).max(self.uv_c.x);

            let bot = self.uv_a.y.min(self.uv_b.y).min(self.uv_c.y);
            let top = self.uv_a.y.max(self.uv_b.y).max(self.uv_c.y);

            let width = right - left;
            let height = top - bot;

            let u_mapped = left + width * u;
            let v_mapped = bot + height * v;

            let uv_hit = Vec2::new(u_mapped, v_mapped);

            Some(Intersection::new(
                intersection_point,
                self.normal,
                dist,
                &self.material,
                is_front_face,
                uv_hit,
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

pub fn load_gltf(file_path: &str, _mesh_material: Arc<Material>) -> Vec<Vec<Triangle>> {
    let (gltf, buffers, images) = gltf::import(file_path)
        .unwrap_or_else(|_| panic!("gltf loader failed to read {}", file_path));
    let mut meshes = Vec::new();

    for mesh in gltf.meshes() {
        // Note: gltf only supports triangles, which is why I only handle tris
        for triangle in mesh.primitives() {
            let reader = triangle.reader(|buffer| Some(&buffers[buffer.index()]));

            let material = triangle.material();

            let mut texture_image = None;

            if let Some(texture_info) = material.pbr_metallic_roughness().base_color_texture() {
                let texture = texture_info.texture();
                let source = texture.source();
                let image = &images[source.index()];
                let im: crate::camera::Image = image.into();

                texture_image = Some(im); // Replace `Image::from` with your image handling method
            }

            let mesh_material = Arc::new(Material::from_gltf(material, texture_image));

            if let (Some(indices), Some(positions)) =
                (reader.read_indices(), reader.read_positions())
            {
                let indices: Vec<u32> = indices.into_u32().collect(); // Convert indices to u32
                let positions: Vec<[f32; 3]> = positions.collect(); // Collect positions
                                                                    //
                let tex_coords: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|coords| coords.into_f32().collect())
                    .expect("no tex coords"); // Read texture coordinates

                let tris: Vec<Triangle> = indices
                    .par_chunks_exact(3)
                    .map(|tri_indices| {
                        let points: Vec<Point3> = tri_indices
                            .iter()
                            .map(|&index| {
                                let pos = positions[index as usize]; // Get position by index
                                Point3::new(
                                    Float::from(pos[0]),
                                    Float::from(pos[1]),
                                    Float::from(pos[2]),
                                )
                            })
                            .collect();

                        let uvs: Vec<Vec2> = tri_indices
                            .iter()
                            .map(|&idx| {
                                let uv = tex_coords[idx as usize];
                                Vec2::new(uv[0] as Float, uv[1] as Float)
                            })
                            .collect();

                        // TODO: make this use new instead of new_with_uv when None
                        // though tbh it doesn't actually matter since textures shouldn't be used for a mesh with no texture map defined
                        Triangle::new_with_uv(
                            points[0],
                            points[1],
                            points[2],
                            uvs[0],
                            uvs[1],
                            uvs[2],
                            mesh_material.clone(),
                        )
                    })
                    .collect();
                meshes.push(tris)
            }
        }
    }
    meshes
}
