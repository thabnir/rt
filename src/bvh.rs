use crate::{
    camera::Float,
    hittable::Hit,
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
    pub fn around(a: AxisAlignedBoundingBox, b: AxisAlignedBoundingBox) -> AxisAlignedBoundingBox {
        AxisAlignedBoundingBox {
            x: range_around(a.x, b.x),
            y: range_around(a.y, b.y),
            z: range_around(a.z, b.z),
        }
    }

    pub fn axes(&self) -> Vec<&Range<Float>> {
        vec![&self.x, &self.y, &self.z]
    }
}

// TODO: should this even be a hittalbe, or should it be a separate boolean function?
// This doesn't actually return a HitRecord, just says whether the ray hits the box
impl Hit for AxisAlignedBoundingBox {
    fn hit(&self, ray: &Ray, range: &Range<Float>) -> Option<HitRecord> {
        for (i, axis) in self.axes().into_iter().enumerate() {
            let ad_inverse = 1.0 / ray.direction[i];
            // direction[axis_index] is what, exactly? if x,y,z should be ok i think?
            // TODO: check the code for the rt in one weekend impl on github and figure out how
            // they're indexing into vec3s for directions like they are

            let t0 = (axis.start - ray.origin[i]) * ad_inverse;
            let t1 = (axis.end - ray.origin[i]) * ad_inverse;

            let t_min = t0.min(t1);
            let t_max = t0.max(t1);

            let range = t_min.max(range.start)..t_max.min(range.end);
            if range.is_empty() {
                return None;
            }
        }

        // TODO: FIX. This is nonsense and should never be used.
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
