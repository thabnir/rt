use crate::camera::{linear_to_gamma, Float};
use glam::Vec3;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::thread_rng;
use rand::Rng;

pub trait Vec3Ext {
    fn as_rgb(&self) -> String;
    fn near_zero(&self) -> bool;
    fn random<R: Rng + ?Sized>(rng: &mut R, min: Float, max: Float) -> Self;
    fn random_unit<R: Rng + ?Sized>(rng: &mut R) -> Self;
    fn random_in_unit_disc<R: Rng + ?Sized>(rng: &mut R) -> Self;
    fn random_on_hemisphere(normal: &Vec3) -> Vec3;
}

impl Vec3Ext for Vec3 {
    fn as_rgb(&self) -> String {
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

        let cmax = 255.999;
        let r = (linear_to_gamma(self.x) * cmax).round() as u8;
        let g = (linear_to_gamma(self.y) * cmax).round() as u8;
        let b = (linear_to_gamma(self.z) * cmax).round() as u8;
        let string = format!("{} {} {}", r, g, b);
        string
    }

    fn random<R: Rng + ?Sized>(rng: &mut R, min: Float, max: Float) -> Self {
        let range = Uniform::from(min..=max);
        Vec3::new(range.sample(rng), range.sample(rng), range.sample(rng))
    }

    fn random_unit<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::random(rng, -1.0, 1.0).normalize()
    }

    /// Returns random point in the x-y unit disc
    fn random_in_unit_disc<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut v = Vec3::ONE;
        let range = -1.0..1.0;
        while v.length_squared() > 1.0 {
            v = Self {
                x: rng.gen_range(range.clone()),
                y: rng.gen_range(range.clone()),
                z: 0.0,
            };
        }
        v
    }

    fn random_on_hemisphere(normal: &Vec3) -> Vec3 {
        let unit_vector: Vec3 = Vec3::random_unit(&mut thread_rng());
        if unit_vector.dot(*normal) > 0.0 {
            return unit_vector; // facing same direction as normal (out from sphere)
        }
        -unit_vector // facing toward center of sphere (must be inverted to reflect)
    }

    fn near_zero(&self) -> bool {
        // Based on https://docs.rs/almost/latest/almost/
        // Which defaults to Float::EPSILON.sqrt() as a comparison
        // to determine if a number is "almost" zero
        let e = Float::EPSILON.sqrt();
        self.x.abs() < e && self.y.abs() < e && self.z.abs() < e
    }
}
