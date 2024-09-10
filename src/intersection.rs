use crate::{
    camera::Float,
    material::Material,
    vec3::{Point3, Ray, Vec2, Vec3},
};

#[derive(Debug)]
pub struct Intersection<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: &'a Material,
    pub t: Float,
    pub is_front_face: bool,
    pub uv: Vec2,
}

impl<'a> Intersection<'a> {
    pub fn new(
        point: Point3,
        normal: Vec3,
        t: Float,
        material: &'a Material,
        is_front_face: bool,
        uv: Vec2,
    ) -> Self {
        Intersection {
            point,
            normal,
            material,
            t,
            is_front_face,
            uv,
        }
    }

    pub fn is_front_face(ray: &Ray, outward_normal: &Vec3) -> bool {
        ray.direction.dot(outward_normal) < 0.0
    }
}
