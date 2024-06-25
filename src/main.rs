pub mod ray;
pub mod vec3;

use crate::ray::{Ray, Sphere};
use crate::vec3::{Color, Vec3};
use indicatif::ProgressBar;
use ray::Hittable;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

type Float = f32;
type UInt = u16;

const T_MIN: f32 = Float::MIN; // maybe 0
const T_MAX: f32 = Float::MAX;

/// Note: in future, this should probably just print to stdout for better terminal piping support
/// For now, since it's easier, it just writes to a file
fn main() -> std::io::Result<()> {
    // Image metadata
    let aspect_ratio: Float = 16.0 / 9.0;
    let img_width: UInt = 2400;
    let img_height = {
        let mut height = (img_width as Float / aspect_ratio) as UInt;
        if height < 1 {
            height = 1;
        }
        height
    };

    // Viewport & Camera
    let focal_length = 0.5;
    let camera_center = Vec3::new(0.0, 0.0, 0.0);
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (img_width as Float) / (img_height as Float);

    // Displacement from left to right and top to bottom of viewport
    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

    // Viewport distance between pixels
    let pixel_du = viewport_u / (img_width as Float);
    let pixel_dv = viewport_v / (img_height as Float);

    let vp_upper_left =
        camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = vp_upper_left + (pixel_du + pixel_dv) / 2.0;

    // Rendering
    let mut buf_writer = BufWriter::new(File::create("out.ppm")?);
    let header = format!(
        "P3\n{} {} # width, height\n255 # max color value\n",
        img_width, img_height
    );
    buf_writer.write_all(header.as_bytes())?;

    println!("Rendering...");
    let progress_bar = ProgressBar::new((img_width as u32 * img_height as u32).into());
    for y in 0..img_height {
        for x in 0..img_width {
            let pixel_center = pixel00_loc + (pixel_du * x as Float) + (pixel_dv * y as Float);
            let ray_dir = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_dir);

            let color = ray_color(&ray);
            let color_str = format!("{} ", color.as_rgb());

            buf_writer.write_all(color_str.as_bytes())?;
            progress_bar.inc(1);
        }
        buf_writer.write_all("\n".as_bytes())?;
    }
    buf_writer.flush()?;
    progress_bar.finish();
    println!("Done.");
    Ok(())
}

fn ray_color(ray: &Ray<Float>) -> Color<Float> {
    let sphere = Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5);
    if let Some(hit_sphere) = sphere.hit(ray, T_MIN, T_MAX) {
        // TODO: UGLY nested ifs!
        if hit_sphere.is_front_face {
            return (hit_sphere.normal + Vec3::one()) / 2.0;
        }
    }
    let unit_dir = ray.direction.normalized();
    let a = (unit_dir.y + 1.0) / 2.0;
    Vec3::new(1.0, 1.0, 1.0) * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
}
