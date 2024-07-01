use num_traits::{Float, One, Zero};
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::camera::linear_to_gamma;

/// x: red, right
///
/// y: green, up
///
/// z: blue, forward
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3<Scalar> {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl<Scalar: Zero> Vec3<Scalar> {
    pub fn zero() -> Self {
        Vec3 {
            x: Scalar::zero(),
            y: Scalar::zero(),
            z: Scalar::zero(),
        }
    }
}

impl<Scalar: One> Vec3<Scalar> {
    pub fn one() -> Self {
        Vec3 {
            x: Scalar::one(),
            y: Scalar::one(),
            z: Scalar::one(),
        }
    }
}

impl<Scalar> Vec3<Scalar> {
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Vec3 { x, y, z }
    }
}

impl<Scalar: SampleUniform + PartialOrd + Copy> Vec3<Scalar> {
    pub fn random<R: Rng + ?Sized>(rng: &mut R, min: Scalar, max: Scalar) -> Self {
        let range = Uniform::from(min..=max);
        Vec3::new(range.sample(rng), range.sample(rng), range.sample(rng))
    }
}

impl<Scalar: Float + SampleUniform> Vec3<Scalar> {
    pub fn random_unit<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::random(rng, -Scalar::one(), Scalar::one()).normalized()
    }
}

impl<Scalar: Add<Output = Scalar> + Sub<Output = Scalar> + Mul<Output = Scalar> + Copy>
    Vec3<Scalar>
{
    pub fn dot(self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length_squared(self) -> Scalar {
        self.dot(self)
    }

    pub fn cross(self, other: Self) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl<Scalar: Float> Vec3<Scalar> {
    pub fn length(self) -> Scalar {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        self / self.length()
    }
}

pub type Color<T> = Vec3<T>;
pub type Point3<T> = Vec3<T>;

/// returns a color string of the form ""
/// Takes a float with x,y,z values between 0.0 and 1.0
impl<Scalar: Float + Display> Color<Scalar> {
    pub fn as_rgb(&self) -> String {
        // if these ever fail the world has ended. still probably best to use better error handling, though
        let cmax = Scalar::from(255.999).unwrap();

        if self.x < Scalar::zero() || self.x > Scalar::one() {
            panic!(
                "Bad color value for red/x: {}. Value should be between 0.0 and 1.0",
                self.x
            );
        }

        if self.y < Scalar::zero() || self.y > Scalar::one() {
            panic!(
                "Bad color value for green/y: {}. Value should be between 0.0 and 1.0",
                self.y
            );
        }

        if self.z < Scalar::zero() || self.z > Scalar::one() {
            panic!(
                "Bad color value for blue/z: {}. Value should be between 0.0 and 1.0",
                self.z
            );
        }
        let r = linear_to_gamma(self.x) * cmax;
        let g = linear_to_gamma(self.y) * cmax;
        let b = linear_to_gamma(self.z) * cmax;
        let ppm_r = r.to_u8().unwrap();
        let ppm_g = g.to_u8().unwrap();
        let ppm_b = b.to_u8().unwrap();
        let string = format!("{} {} {}", ppm_r, ppm_g, ppm_b);
        string
    }
}

impl<Scalar: Add<Output = Scalar>> Add for Vec3<Scalar> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<Scalar: Add<Output = Scalar> + Zero> Sum for Vec3<Scalar> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |a, b| Self {
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
        })
    }
}

impl<Scalar: Sub<Output = Scalar>> Sub for Vec3<Scalar> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<Scalar: Div<Output = Scalar> + Copy> Div<Scalar> for Vec3<Scalar> {
    type Output = Self;

    fn div(self, scalar: Scalar) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl<Scalar: Mul<Output = Scalar> + Copy> Mul<Scalar> for Vec3<Scalar> {
    type Output = Self;

    fn mul(self, scalar: Scalar) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl<Scalar: Add<Output = Scalar> + Copy> AddAssign for Vec3<Scalar> {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
        self.z = self.z + rhs.z;
    }
}

impl<Scalar: Sub<Output = Scalar> + Copy> SubAssign for Vec3<Scalar> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
        self.z = self.z - rhs.z;
    }
}

impl<Scalar: Mul<Output = Scalar> + Copy> MulAssign for Vec3<Scalar> {
    fn mul_assign(&mut self, rhs: Self) {
        self.x = self.x * rhs.x;
        self.y = self.y * rhs.y;
        self.z = self.z * rhs.z;
    }
}

impl<Scalar: Div<Output = Scalar> + Copy> DivAssign for Vec3<Scalar> {
    fn div_assign(&mut self, rhs: Self) {
        self.x = self.x / rhs.x;
        self.y = self.y / rhs.y;
        self.z = self.z / rhs.z;
    }
}

impl<Scalar: Neg<Output = Scalar>> Neg for Vec3<Scalar> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_new() {
        let v: Vec3<f64> = Vec3::new(1.0, 2.0, 3.0);
        assert_abs_diff_eq!(v.x, 1.0);
        assert_abs_diff_eq!(v.y, 2.0);
        assert_abs_diff_eq!(v.z, 3.0);
    }

    #[test]
    fn test_dot() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_abs_diff_eq!(v1.dot(v2), 32.0);
    }

    #[test]
    fn test_length() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_abs_diff_eq!(v.length(), 14.0.sqrt());
    }

    #[test]
    fn test_normalize() {
        let v: Vec3<f32> = Vec3::new(3.0, 4.0, 5.0);
        let normalized = v.normalized();
        assert_abs_diff_eq!(normalized.length(), 1.0);
    }

    #[test]
    fn test_normalize_2() {
        let v: Vec3<f64> = Vec3::new(3.0, 4.0, 5.0);
        let normalized = v.normalized();
        assert_abs_diff_eq!(normalized.length(), 1.0);
    }

    #[test]
    fn test_zero() {
        let zero_vec: Vec3<f64> = Vec3::zero();
        assert_abs_diff_eq!(zero_vec.x, 0.0);
        assert_abs_diff_eq!(zero_vec.y, 0.0);
        assert_abs_diff_eq!(zero_vec.z, 0.0);
    }

    #[test]
    fn test_one() {
        let one_vec: Vec3<f64> = Vec3::one();
        assert_abs_diff_eq!(one_vec.x, 1.0);
        assert_abs_diff_eq!(one_vec.y, 1.0);
        assert_abs_diff_eq!(one_vec.z, 1.0);
    }

    #[test]
    fn test_subtraction() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1 - v2;
        assert_abs_diff_eq!(result.x, -3.0);
        assert_abs_diff_eq!(result.y, -3.0);
        assert_abs_diff_eq!(result.z, -3.0);
    }

    #[test]
    fn test_division() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let scalar = 2.0;
        let result = v / scalar;
        assert_abs_diff_eq!(result.x, 1.0);
        assert_abs_diff_eq!(result.y, 2.0);
        assert_abs_diff_eq!(result.z, 3.0);
    }

    #[test]
    fn test_multiplication() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let scalar = 2.0;
        let result = v * scalar;
        assert_abs_diff_eq!(result.x, 2.0);
        assert_abs_diff_eq!(result.y, 4.0);
        assert_abs_diff_eq!(result.z, 6.0);
    }

    #[test]
    fn test_addition_assignment() {
        let mut v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        v1 += v2;
        assert_abs_diff_eq!(v1.x, 5.0);
        assert_abs_diff_eq!(v1.y, 7.0);
        assert_abs_diff_eq!(v1.z, 9.0);
    }

    #[test]
    fn test_subtraction_assignment() {
        let mut v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        v1 -= v2;
        assert_abs_diff_eq!(v1.x, -3.0);
        assert_abs_diff_eq!(v1.y, -3.0);
        assert_abs_diff_eq!(v1.z, -3.0);
    }

    #[test]
    fn test_multiplication_assignment() {
        let mut v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(2.0, 3.0, 4.0);
        v1 *= v2;
        assert_abs_diff_eq!(v1.x, 2.0);
        assert_abs_diff_eq!(v1.y, 6.0);
        assert_abs_diff_eq!(v1.z, 12.0);
    }

    #[test]
    fn test_division_assignment() {
        let mut v1 = Vec3::new(2.0, 4.0, 6.0);
        let v2 = Vec3::new(2.0, 2.0, 2.0);
        v1 /= v2;
        assert_abs_diff_eq!(v1.x, 1.0);
        assert_abs_diff_eq!(v1.y, 2.0);
        assert_abs_diff_eq!(v1.z, 3.0);
    }

    #[test]
    fn test_negation() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let negated = -v;
        assert_abs_diff_eq!(negated.x, -1.0);
        assert_abs_diff_eq!(negated.y, -2.0);
        assert_abs_diff_eq!(negated.z, -3.0);
    }

    #[test]
    fn test_cross_product() {
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let result = v1.cross(v2);
        assert_abs_diff_eq!(result.x, 0.0);
        assert_abs_diff_eq!(result.y, 0.0);
        assert_abs_diff_eq!(result.z, 1.0);
    }

    #[test]
    fn test_cross_product_parallel() {
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(2.0, 0.0, 0.0);
        let result = v1.cross(v2);
        assert_abs_diff_eq!(result.x, 0.0);
        assert_abs_diff_eq!(result.y, 0.0);
        assert_abs_diff_eq!(result.z, 0.0);
    }

    #[test]
    fn test_cross_product_opposite() {
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(-1.0, 0.0, 0.0);
        let result = v1.cross(v2);
        assert_abs_diff_eq!(result.x, 0.0);
        assert_abs_diff_eq!(result.y, 0.0);
        assert_abs_diff_eq!(result.z, 0.0);
    }
}
