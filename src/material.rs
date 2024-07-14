use crate::{
    camera::Float,
    ray::{HitRecord, Ray},
    vec3_ext::Vec3Ext,
};
use glam::Vec3;
use rand::{thread_rng, Rng};

pub trait Scatter: Send + Sync {
    fn scatter(&self, ray_in: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)>;
}

fn reflect(incoming_direction: Vec3, surface_normal: Vec3) -> Vec3 {
    // Scale normal by length of incoming ray's direction projected onto the normal
    // Then reflect the ray by subtracting twice the inverse of its height relative to the surface
    let scaled_normal = surface_normal * incoming_direction.dot(surface_normal);
    incoming_direction - scaled_normal * 2.0
}

/// Expects `incoming_direction` to be a unit vector
fn refract(incoming_direction: Vec3, surface_normal: Vec3, refractive_ratio: Float) -> Vec3 {
    let cos_theta = (-incoming_direction.dot(surface_normal)).min(1.0);
    let r_out_perp = (incoming_direction + surface_normal * cos_theta) * refractive_ratio;
    let x = -((1.0 - r_out_perp.length_squared()).abs().sqrt());
    let r_out_parallel = surface_normal * x;
    r_out_parallel + r_out_perp
}

#[derive(Clone, Copy)]
pub struct Lambertian {
    pub albedo: Vec3,
}

#[derive(Clone, Copy)]
pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: Float,
}

impl Scatter for Metal {
    fn scatter(&self, ray_in: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)> {
        let reflected = reflect(ray_in.direction, record.normal)
            + Vec3::random_unit(&mut thread_rng()) * self.fuzz;
        let scattered = Ray::new(record.point, reflected);
        Some((self.albedo, scattered))
    }
}

impl Scatter for Lambertian {
    fn scatter(&self, _ray_in: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)> {
        let mut scatter_dir = hit.normal + Vec3::random_unit(&mut thread_rng());
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }
        let scattered = Ray::new(hit.point, scatter_dir);
        Some((self.albedo, scattered))
    }
}

#[derive(Clone, Copy)]
pub struct Dielectric {
    /// Refractive index in vacuum or air, or the ratio of the material's
    /// refractive index over the refractive index of the enclosing media
    pub refractive_index: Float,
    // albedo: Scalar, // TBD if this is needed (how to implement colored transparents?)
}

impl Scatter for Dielectric {
    fn scatter(&self, ray_in: &Ray, record: &HitRecord) -> Option<(Vec3, Ray)> {
        let ri = if record.is_front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };

        let incoming_direction = ray_in.direction.normalize();

        let cos_theta = (-incoming_direction.dot(record.normal)).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt(); // sin^2(x) + cos^2(x) = 1
        let cannot_refract = ri * sin_theta > 1.0;

        let noise = thread_rng().gen_range(0.0..1.0);

        let direction = if cannot_refract || reflectance(cos_theta, ri) > noise {
            reflect(incoming_direction, record.normal)
        } else {
            refract(incoming_direction, record.normal, ri)
        };
        Some((Vec3::ONE, Ray::new(record.point, direction)))
    }
}

/// Returns Schlick's approximation for reflectance at a given angle.
fn reflectance(cosine: Float, refractive_index: Float) -> Float {
    let r0 = (1.0 - refractive_index) / (1.0 + refractive_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}