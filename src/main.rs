pub mod ray;
pub mod vec3;

use crate::ray::Ray;
use crate::vec3::{Color, Point, Vec3};
use indicatif::ProgressBar;
use std::fs::File;
use std::io::prelude::*;

type Float = f64;

/// Note: in future, this should probably just print to stdout for better terminal piping support
/// For now, since it's easier, it just writes to a file
fn main() -> std::io::Result<()> {
    // Image metadata
    let aspect_ratio: Float = 16.0 / 9.0;
    let img_width: u32 = 800;
    let img_height = {
        let mut height = (img_width as Float / aspect_ratio) as u32;
        if height < 1 {
            height = 1;
        }
        height
    };

    // Viewport & Camera
    let focal_length = 1.0;
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
    let pixel00_loc = vp_upper_left + (pixel_du + pixel_dv) * 0.5;

    // Rendering
    let mut file = File::create("out.ppm")?;
    let header = format!(
        "P3\n{} {} # width, height\n255 # max color value\n",
        img_width, img_height
    );
    file.write_all(header.as_bytes())?;

    println!("Rendering...");
    let progress_bar = ProgressBar::new((img_width * img_height).into());
    for y in 0..img_height {
        for x in 0..img_width {
            let pixel_center = pixel00_loc + (pixel_du * x.into()) + (pixel_dv * y.into());
            let ray_dir = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_dir);

            let color = ray_color(&ray);
            // let color = Vec3::new(1.0, 0.0, 0.0);
            let color_str = format!("{} ", color.as_rgb());

            file.write_all(color_str.as_bytes())?;
            progress_bar.inc(1);
        }
        file.write_all("\n".as_bytes())?;
    }
    progress_bar.finish();
    println!("Done.");
    Ok(())
}

fn ray_color(ray: &Ray<Float>) -> Color<Float> {
    if hit_sphere(Vec3::new(0.0, 0.0, -1.0), 0.5, &ray) {
        return Vec3::new(1.0, 0.0, 0.0);
    }
    let unit_dir = ray.direction.normalized();
    let a = (unit_dir.y + 1.0) * 0.5;
    return Vec3::new(1.0, 1.0, 1.0) * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a;
}

fn hit_sphere(center: Point<Float>, radius: Float, ray: &Ray<Float>) -> bool {
    let oc = center - ray.origin;
    // let a = ray.direction.dot(&ray.direction);
    let a = ray.direction.length_squared();
    let b = ray.direction.dot(&oc) * -2.0;
    // let c = oc.dot(&oc) - radius * radius;
    let c = oc.length_squared() - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
}
