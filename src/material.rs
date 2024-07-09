use num_traits::Float;
use rand::{distributions::uniform::SampleUniform, thread_rng, Rng};

use crate::{
    ray::{HitRecord, Ray},
    vec3::{Color, Vec3},
};

pub trait Scatter<Scalar: Copy + Clone>: Send + Sync {
    fn scatter(
        &self,
        ray_in: &Ray<Scalar>,
        record: &HitRecord<Scalar>,
    ) -> Option<(Color<Scalar>, Ray<Scalar>)>;
}

fn reflect<Scalar: Float + Send + Sync>(
    incoming_direction: Vec3<Scalar>,
    surface_normal: Vec3<Scalar>,
) -> Vec3<Scalar> {
    // Scale normal by length of incoming ray's direction projected onto the normal
    // Then reflect the ray by subtracting twice the inverse of its height relative to the surface
    let scaled_normal = surface_normal * incoming_direction.dot(surface_normal);
    incoming_direction - scaled_normal * Scalar::from(2.0).unwrap()
}

/// Expects `incoming_direction` to be a unit vector
fn refract<Scalar: Float + std::fmt::Debug + Send + Sync>(
    incoming_direction: Vec3<Scalar>,
    surface_normal: Vec3<Scalar>,
    refractive_ratio: Scalar,
) -> Vec3<Scalar> {
    let cos_theta = (-incoming_direction.dot(surface_normal)).min(Scalar::one());
    let r_out_perp = (incoming_direction + surface_normal * cos_theta) * refractive_ratio;
    let x = -((Scalar::one() - r_out_perp.length_squared()).abs().sqrt());
    let r_out_parallel = surface_normal * x;
    r_out_parallel + r_out_perp
}

#[derive(Clone, Copy)]
pub struct Lambertian<Scalar> {
    pub albedo: Color<Scalar>,
}

#[derive(Clone, Copy)]
pub struct Metal<Scalar> {
    pub albedo: Color<Scalar>,
    pub fuzz: Scalar,
}

impl<Scalar: Float + SampleUniform + Send + Sync> Scatter<Scalar> for Metal<Scalar> {
    fn scatter(
        &self,
        ray_in: &Ray<Scalar>,
        record: &HitRecord<Scalar>,
    ) -> Option<(Color<Scalar>, Ray<Scalar>)> {
        let reflected = reflect(ray_in.direction, record.normal)
            + Vec3::random_unit(&mut thread_rng()) * self.fuzz;
        let scattered = Ray::new(record.point, reflected);
        Some((self.albedo, scattered))
    }
}

impl<Scalar: Float + SampleUniform + Send + Sync> Scatter<Scalar> for Lambertian<Scalar> {
    fn scatter(
        &self,
        _ray_in: &Ray<Scalar>,
        hit: &HitRecord<Scalar>,
    ) -> Option<(Color<Scalar>, Ray<Scalar>)> {
        let mut scatter_dir = hit.normal + Vec3::random_unit(&mut thread_rng());
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }
        let scattered = Ray::new(hit.point, scatter_dir);
        Some((self.albedo, scattered))
    }
}

#[derive(Clone, Copy)]
pub struct Dielectric<Scalar> {
    /// Refractive index in vacuum or air, or the ratio of the material's
    /// refractive index over the refractive index of the enclosing media
    pub refractive_index: Scalar,
    // albedo: Scalar, // TBD if this is needed (how to implement colored transparents?)
}

impl<Scalar: Float + SampleUniform + Send + Sync + std::fmt::Debug> Scatter<Scalar>
    for Dielectric<Scalar>
{
    fn scatter(
        &self,
        ray_in: &Ray<Scalar>,
        record: &HitRecord<Scalar>,
    ) -> Option<(Color<Scalar>, Ray<Scalar>)> {
        let ri = if record.is_front_face {
            Scalar::one() / self.refractive_index
        } else {
            self.refractive_index
        };

        let incoming_direction = ray_in.direction.normalized();

        let cos_theta = (-incoming_direction.dot(record.normal)).min(Scalar::one());
        let sin_theta = (Scalar::one() - cos_theta * cos_theta).sqrt(); // sin^2(x) + cos^2(x) = 1
        let cannot_refract = ri * sin_theta > Scalar::one();

        let noise = thread_rng().gen_range(Scalar::zero()..Scalar::one());

        let direction = if cannot_refract || reflectance(cos_theta, ri) > noise {
            reflect(incoming_direction, record.normal)
        } else {
            refract(incoming_direction, record.normal, ri)
        };
        Some((Color::one(), Ray::new(record.point, direction)))
    }
}

/// Returns Schlick's approximation for reflectance at a given angle.
fn reflectance<Scalar: Float + Send + Sync>(cosine: Scalar, refractive_index: Scalar) -> Scalar {
    let r0 = (Scalar::one() - refractive_index) / (Scalar::one() + refractive_index);
    let r0 = r0 * r0;
    r0 + (Scalar::one() - r0) * (Scalar::one() - cosine).powi(5)
}
