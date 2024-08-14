use crate::{
    camera::Float,
    material::Material,
    vec3::{Point3, Ray, Vec3},
};

#[derive(Clone)]
pub struct Intersection {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Material,
    pub t: Float,
    pub is_front_face: bool,
}

impl Intersection {
    pub fn new(
        point: Point3,
        normal: Vec3,
        t: Float,
        material: Material,
        is_front_face: bool,
    ) -> Self {
        Intersection {
            point,
            normal,
            material,
            t,
            is_front_face,
        }
    }

    pub fn is_front_face(ray: &Ray, outward_normal: &Vec3) -> bool {
        ray.direction.dot(outward_normal) < 0.0
    }
}
