use crate::vec3::{Point, Vec3};
use num_traits::Float;

pub struct Ray<Scalar: Float> {
    pub origin: Point<Scalar>,
    pub direction: Vec3<Scalar>,
}

impl<Scalar: Float> Ray<Scalar> {
    pub fn new(origin: Point<Scalar>, direction: Vec3<Scalar>) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, time: Scalar) -> Vec3<Scalar> {
        self.origin + self.direction * time
    }
}
