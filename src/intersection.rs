use crate::{
    camera::Float,
    material::Material,
    vec3::{Point3, Ray, Vec3},
};

pub struct Intersection<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: &'a Material,
    pub t: Float,
    pub is_front_face: bool,
    pub u: Float,
    pub v: Float,
}

impl<'a> Intersection<'a> {
    pub fn new(
        point: Point3,
        normal: Vec3,
        t: Float,
        material: &'a Material,
        is_front_face: bool,
        u: Float,
        v: Float,
    ) -> Self {
        Intersection {
            point,
            normal,
            material,
            t,
            is_front_face,
            u,
            v,
        }
    }

    pub fn is_front_face(ray: &Ray, outward_normal: &Vec3) -> bool {
        ray.direction.dot(outward_normal) < 0.0
    }
}
