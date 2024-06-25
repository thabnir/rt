use num_traits::Float;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// x: red, right
///
/// y: green, up
///
/// z: blue, forward
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Vec3<Scalar: Float> {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl<Scalar: Float> Vec3<Scalar> {
    pub fn zero() -> Self {
        Vec3 {
            x: Scalar::zero(),
            y: Scalar::zero(),
            z: Scalar::zero(),
        }
    }

    pub fn one() -> Self {
        Vec3 {
            x: Scalar::one(),
            y: Scalar::one(),
            z: Scalar::one(),
        }
    }

    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Vec3 { x, y, z }
    }

    pub fn dot(&self, other: &Self) -> Scalar {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalized(&self) -> Self {
        *self / self.length()
    }
}

pub type Color<Scalar> = Vec3<Scalar>;
pub type Point<Scalar> = Vec3<Scalar>;

impl<Scalar: Float + Display> Color<Scalar> {
    pub fn as_rgb(&self) -> String {
        let cmax = Scalar::from(255.999).unwrap();
        let one = Scalar::from(1.0).unwrap();

        if self.x < Scalar::neg_zero() || self.x > one {
            panic!(
                "Bad color value for red/x: {}. Value should be between 0.0 and 1.0",
                self.x
            );
        }

        if self.y < Scalar::neg_zero() || self.y > one {
            panic!(
                "Bad color value for green/y: {}. Value should be between 0.0 and 1.0",
                self.y
            );
        }

        if self.z < Scalar::neg_zero() || self.z > one {
            panic!(
                "Bad color value for blue/z: {}. Value should be between 0.0 and 1.0",
                self.z
            );
        }
        let ppm_r = Scalar::from(self.x * cmax).unwrap().to_u8().unwrap();
        let ppm_g = Scalar::from(self.y * cmax).unwrap().to_u8().unwrap();
        let ppm_b = Scalar::from(self.z * cmax).unwrap().to_u8().unwrap();
        let string = format!("{} {} {}", ppm_r, ppm_g, ppm_b);
        string
    }
}

impl<Scalar: Float> Add for Vec3<Scalar> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<Scalar: Float> Sub for Vec3<Scalar> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<Scalar: Float> Div<Scalar> for Vec3<Scalar> {
    type Output = Self;

    fn div(self, scalar: Scalar) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl<VecType: Float> Mul<VecType> for Vec3<VecType> {
    type Output = Self;

    fn mul(self, scalar: VecType) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl<Scalar: Float> AddAssign for Vec3<Scalar> {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
        self.z = self.z + rhs.z;
    }
}

impl<Scalar: Float> SubAssign for Vec3<Scalar> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x = self.x - rhs.x;
        self.y = self.y - rhs.y;
        self.z = self.z - rhs.z;
    }
}

impl<Scalar: Float> MulAssign for Vec3<Scalar> {
    fn mul_assign(&mut self, rhs: Self) {
        self.x = self.x * rhs.x;
        self.y = self.y * rhs.y;
        self.z = self.z * rhs.z;
    }
}

impl<Scalar: Float> DivAssign for Vec3<Scalar> {
    fn div_assign(&mut self, rhs: Self) {
        self.x = self.x / rhs.x;
        self.y = self.y / rhs.y;
        self.z = self.z / rhs.z;
    }
}

impl<Scalar: Float> Neg for Vec3<Scalar> {
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
        assert_abs_diff_eq!(v1.dot(&v2), 32.0);
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
}
