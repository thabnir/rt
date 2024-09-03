use crate::camera::Float;
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use rand::Rng;

pub type Ray = bvh::ray::Ray<Float, 3>;
pub type Vec3 = nalgebra::Vector3<Float>;
// pub type Point3 = nalgebra::Point3<Float>;
pub type Point3 = nalgebra::Vector3<Float>; // TODO: make this use Point3 instead

pub trait RayExt {
    fn at(&self, time: Float) -> Vec3;
}

impl RayExt for Ray {
    fn at(&self, time: Float) -> Vec3 {
        self.direction * time + self.origin.coords
    }
}

pub trait Vec3Ext {
    const ONE: Vec3;
    fn as_gamma_vec(&self) -> Vec3;
    fn as_rgb_linear(&self) -> (u8, u8, u8);
    fn as_rgb_gamma(&self) -> (u8, u8, u8);
    fn as_rgb_gamma_string(&self) -> String;
    fn near_zero(&self) -> bool;
    fn random<R: Rng + ?Sized>(rng: &mut R, min: Float, max: Float) -> Self;
    fn random_unit<R: Rng + ?Sized>(rng: &mut R) -> Self;
    fn random_in_unit_disc<R: Rng + ?Sized>(rng: &mut R) -> Self;
    fn random_on_hemisphere(normal: &Vec3) -> Vec3;
}

impl Vec3Ext for Vec3 {
    const ONE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

    /// Takes a color in linear space with values from 0.0 to 1.0 and gamma corrects it
    fn as_gamma_vec(&self) -> Vec3 {
        let gamma = 1.0 / 2.2;
        Vec3::new(self.x.powf(gamma), self.y.powf(gamma), self.z.powf(gamma))
    }

    fn as_rgb_linear(&self) -> (u8, u8, u8) {
        let color_range = 0.0..=1.0;
        if !color_range.contains(&self.x) {
            panic!(
                "Bad color value for red/x: {}. Value should be between 0.0 and 1.0",
                self.x
            );
        }
        if !color_range.contains(&self.y) {
            panic!(
                "Bad color value for green/y: {}. Value should be between 0.0 and 1.0",
                self.y
            );
        }
        if !color_range.contains(&self.z) {
            panic!(
                "Bad color value for blue/z: {}. Value should be between 0.0 and 1.0",
                self.z
            );
        }
        let cmax = 255.0;
        let r = (self.x * cmax).round() as u8;
        let g = (self.y * cmax).round() as u8;
        let b = (self.z * cmax).round() as u8;
        (r, g, b)
    }

    fn as_rgb_gamma(&self) -> (u8, u8, u8) {
        let color_range = 0.0..=1.0;
        if !color_range.contains(&self.x) {
            panic!(
                "Bad color value for red/x: {}. Value should be between 0.0 and 1.0",
                self.x
            );
        }
        if !color_range.contains(&self.y) {
            panic!(
                "Bad color value for green/y: {}. Value should be between 0.0 and 1.0",
                self.y
            );
        }
        if !color_range.contains(&self.z) {
            panic!(
                "Bad color value for blue/z: {}. Value should be between 0.0 and 1.0",
                self.z
            );
        }
        let cmax = 255.0;
        let gamma_corrected = self.as_gamma_vec();
        let r = (gamma_corrected.x * cmax).round() as u8;
        let g = (gamma_corrected.y * cmax).round() as u8;
        let b = (gamma_corrected.z * cmax).round() as u8;
        (r, g, b)
    }

    fn as_rgb_gamma_string(&self) -> String {
        let (r, g, b) = self.as_rgb_gamma();
        let string = format!("{} {} {}", r, g, b);
        string
    }

    fn near_zero(&self) -> bool {
        // Based on https://docs.rs/almost/latest/almost/
        // Which defaults to Float::EPSILON.sqrt() as a comparison
        // to determine if a number is "almost" zero
        let e = Float::EPSILON.sqrt();
        self.x.abs() < e && self.y.abs() < e && self.z.abs() < e
    }

    fn random<R: Rng + ?Sized>(rng: &mut R, min: Float, max: Float) -> Self {
        let range = Uniform::from(min..=max);
        Vec3::new(range.sample(rng), range.sample(rng), range.sample(rng))
    }

    fn random_unit<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::random(rng, -1.0, 1.0).normalize()
    }

    // TODO: make this not actually random (QMC sampling)
    /// Returns random point in the x-y unit disc
    fn random_in_unit_disc<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut v = Vec3::ONE;
        let range = -1.0..1.0;
        while v.norm_squared() > 1.0 {
            v = Self::new(
                rng.gen_range(range.clone()),
                rng.gen_range(range.clone()),
                0.0,
            );
        }
        v
    }

    /// Returns a random vector in the unit hemisphere with the input `normal` as its pole
    fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
        let unit_vector: Vec3 = Vec3::random_unit(&mut thread_rng());
        if unit_vector.dot(normal) > 0.0 {
            unit_vector
        } else {
            -unit_vector
        }
    }
}
