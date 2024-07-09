pub mod camera;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod vec3;

use camera::{Camera, Float};
use hittable::{Sphere, World};
use indicatif::ProgressBar;
use material::{Dielectric, Lambertian, Metal};
use rand::{thread_rng, Rng};
use std::fs::File;
use std::time::Instant;
use vec3::{Color, Point3, Vec3};

#[allow(dead_code)]
fn gen_scene(num_spheres: u16, min_radius: Float, max_radius: Float) -> World<Float> {
    // TODO: make this thing not suck! No sphere collisions! More reasonable spacing!
    let mut rng = thread_rng();
    let mut world: World<Float> = Vec::new();
    // let lambert = Lambertian {
    //     albedo: Color::new(0.6, 0.6, 0.6),
    // };
    // let ground = Box::new(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0, lambert));
    // world.push(ground);
    for _ in 0..num_spheres {
        let radius = rng.gen_range(min_radius..=max_radius);
        let min_pos = -50.0;
        let max_pos = 50.0;
        let albedo: Color<Float> = Color::rand_color(&mut rng);
        let mut position: Point3<Float> = Vec3::random(&mut rng, min_pos, max_pos);
        while position.distance(Vec3::zero()) < radius + 2.0 {
            position = Vec3::random(&mut rng, min_pos, max_pos);
            position.z = -position.z.abs();
        }
        if rng.gen_bool(0.5) {
            let mat = Lambertian { albedo };
            let sphere = Box::new(Sphere::new(position, radius, mat));
            world.push(sphere);
        } else {
            let fuzz = rng.gen_range(0.0..1.0);
            let mat = Metal { albedo, fuzz };
            let sphere = Box::new(Sphere::new(position, radius, mat));
            world.push(sphere);
        };
    }
    world
}

fn main() -> std::io::Result<()> {
    let mut world: World<Float> = Vec::new();

    let material_ground = Lambertian {
        albedo: Color::new(0.8, 0.8, 0.0),
    };
    let material_center = Lambertian {
        albedo: Color::new(0.1, 0.2, 0.5),
    };
    let material_right = Metal {
        albedo: Color::new(0.8, 0.6, 0.2),
        fuzz: 0.6,
    };
    let glass = Dielectric {
        refractive_index: 1.5,
    };
    let bubble = Dielectric {
        refractive_index: 1.0 / 1.5,
    };
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, -1.2),
        0.5,
        material_center,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.5,
        glass,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(-1.0, 0.0, -1.0),
        0.4,
        bubble,
    )));
    world.push(Box::new(Sphere::new(
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    )));

    let image_width = 800;
    let image_height = 600;
    let samples_per_pixel = 100;
    let max_depth = 100;
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
