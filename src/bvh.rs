use crate::{
    camera::Float,
    hittable::{Hit, Hittable},
    material::{Lambertian, Material},
    ray::{HitRecord, Ray},
    vec3::Vec3,
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct AxisAlignedBoundingBox {
    x: Range<Float>,
    y: Range<Float>,
    z: Range<Float>,
}

// TODO: figure out Rustiest way to implement a BVH structure
// Maybe just treat it like a binary tree?
// Worth checking out https://docs.rs/bvh/latest/bvh/index.html
pub struct BVH {
    children: Vec<Hittable>,
    bounding_box: AxisAlignedBoundingBox,
}

impl BVH {
    pub fn left(&self) -> &Hittable {
        &self.children[0]
    }
    pub fn right(&self) -> &Hittable {
        &self.children[1]
    }
}

impl Hit for BVH {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        if let Some(hr) = self.bounding_box.hit(ray, range) {}
        None
    }

    /// Returns the bounding box surrounding all child nodes
    fn bounding_box(&self) -> &AxisAlignedBoundingBox {
        &self.bounding_box
    }
}

/// Returns the range surrounding r1 and r2
fn range_around(r1: Range<Float>, r2: Range<Float>) -> Range<Float> {
    // TODO: determine if this is correct in the case a range is empty (start >= end)
    let min = r1.start.min(r2.start);
    let max = r1.end.max(r2.end);
    min..max
}

impl AxisAlignedBoundingBox {
    pub fn new(x_range: Range<Float>, y_range: Range<Float>, z_range: Range<Float>) -> Self {
        AxisAlignedBoundingBox {
            x: x_range,
            y: y_range,
            z: z_range,
        }
    }
    pub fn new_from_points(a: Vec3, b: Vec3) -> Self {
        AxisAlignedBoundingBox {
            x: if a.x <= b.x { a.x..b.x } else { b.x..a.x },
            y: if a.y <= b.y { a.y..b.y } else { b.y..a.y },
            z: if a.z <= b.z { a.z..b.z } else { b.z..a.z },
        }
    }

    /// Returns the bounding box that contains/surrounds both input boxes `a` and `b`
    pub fn around(
        a: &AxisAlignedBoundingBox,
        b: &AxisAlignedBoundingBox,
    ) -> AxisAlignedBoundingBox {
        let a = a.clone();
        let b = b.clone();
        AxisAlignedBoundingBox {
            x: range_around(a.x, b.x),
            y: range_around(a.y, b.y),
            z: range_around(a.z, b.z),
        }
    }

    pub fn axes(&self) -> Vec<&Range<Float>> {
        vec![&self.x, &self.y, &self.z]
    }

    /// Bounding box with size 0
    pub const ZERO: Self = AxisAlignedBoundingBox {
        x: 0.0..0.0,
        y: 0.0..0.0,
        z: 0.0..0.0,
    };
}

// TODO: should this even be a hittalbe, or should it just have a separate boolean function?
// This doesn't actually return a HitRecord, just says whether the ray hits the box
impl Hit for AxisAlignedBoundingBox {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        for (i, axis) in self.axes().into_iter().enumerate() {
            // Note: example_vec3[0, 1, 2] = x, y, z
            let ad_inverse = 1.0 / ray.direction[i];
            let t0 = (axis.start - ray.origin[i]) * ad_inverse;
            let t1 = (axis.end - ray.origin[i]) * ad_inverse;

            let t_min = t0.min(t1);
            let t_max = t0.max(t1);

            let range = t_min.max(range.start)..t_max.min(range.end);
            if range.is_empty() {
                return None;
            }
        }

        // TODO: FIX. This is nonsense and should never be used. Make a new `hit` function that returns a boolean.
        // Decide how to update the interfaces accordingly
        Some(HitRecord {
            point: Vec3::default(),
            normal: Vec3::default(),
            material: Material::Lambertian(Lambertian {
                albedo: Vec3::default(),
            }),
            t: 0.0,
            is_front_face: true,
        })
    }

    fn bounding_box(&self) -> &AxisAlignedBoundingBox {
        self
    }
}
