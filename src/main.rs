pub mod camera;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod vec3;

use camera::{Camera, Float};
use hittable::{Sphere, World};
use indicatif::ProgressBar;
use std::fs::File;
use std::time::Instant;
use vec3::Vec3;

fn main() -> std::io::Result<()> {
    let mut world: World<Float> = Vec::new();
    let sphere = Box::new(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5));
    let ground = Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0));
    world.push(sphere);
    world.push(ground);

    let image_width = 800;
    let image_height = 600;
    let samples_per_pixel = 4000;
    let max_depth = 50;
    let camera = Camera::new(
        Vec3::new(0.0, 0.0, 0.0),
        0.5,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
        2.0,
        0.0..Float::MAX,
    );

    let render_progress = ProgressBar::new(image_height as u64);
    println!("Rendering...");
    let now = Instant::now();
    let image = camera.render(&world, render_progress);
    let elapsed = now.elapsed();
    let pixels_per_sec =
        (image_width as Float * image_height as Float) / elapsed.as_millis() as Float;
    println!(
        "Done rendering in {:.2?}, at a rate of {:.0} pixels/ms",
        elapsed, pixels_per_sec
    );

    let out_file = File::create("out.ppm")?;
    let write_progress = ProgressBar::new(image_width as u64 * image_height as u64);
    println!("Writing image to disk...");
    Camera::write_image(image, out_file, Some(write_progress))?;
    println!("Done.");
    Ok(())
}
