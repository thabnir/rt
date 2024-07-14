pub mod camera;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod vec3_ext;

use camera::{Camera, Float};
use glam::Vec3;
use hittable::{Sphere, World};
use indicatif::ProgressBar;
use material::{Dielectric, Lambertian, Metal};
use rand::{thread_rng, Rng};
use std::fs::File;
use std::time::Instant;
use vec3_ext::Vec3Ext;

fn gen_scene(grid_i: i16, grid_j: i16) -> World {
    let mut rng = thread_rng();
    let mut world: World = Vec::new();
    let ground_mat = Lambertian {
        albedo: Vec3::new(0.5, 0.5, 0.5),
    };
    let ground = Box::new(Sphere::new(
        Vec3::new(0.0, -1000.0, -1.0),
        1000.0,
        ground_mat,
    ));
    world.push(ground);
    let mat1 = Dielectric {
        refractive_index: 1.5,
    };
    let p1 = Vec3::new(0.0, 1.0, 0.0);
    world.push(Box::new(Sphere::new(p1, 1.0, mat1)));
    let mat2 = Lambertian {
        albedo: Vec3::new(0.4, 0.2, 0.1),
    };
    let p2 = Vec3::new(-4.0, 1.0, 0.0);
    world.push(Box::new(Sphere::new(p2, 1.0, mat2)));
    let mat3 = Metal {
        albedo: Vec3::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    };
    let p3 = Vec3::new(4.0, 1.0, 0.0);
    world.push(Box::new(Sphere::new(p3, 1.0, mat3)));

    for i in -grid_i..grid_i {
        for j in -grid_j..grid_j {
            let radius = 0.2;
            let albedo: Vec3 = Vec3::random(&mut rng, 0.0, 1.0);
            let offset: Vec3 = Vec3 {
                x: rng.gen_range(0.0..0.9),
                y: 0.0,
                z: rng.gen_range(0.0..0.9),
            };
            let i_offset = 1.0;
            let j_offset = 1.0;
            let center = Vec3::new(i as Float * i_offset, radius, j as Float * j_offset) + offset;
            if center.distance(p1) < 1.2 || center.distance(p2) < 1.2 || center.distance(p3) < 1.2 {
                continue;
            }
            let choose = rng.gen_range(0.0..1.0);
            if choose > (0.95) {
                let mat = Dielectric {
                    refractive_index: 1.5,
                };
                let sphere = Box::new(Sphere::new(center, radius, mat));
                world.push(sphere);
            } else if choose > 0.8 {
                let fuzz = rng.gen_range(0.0..0.5);
                let mat = Metal { albedo, fuzz };
                let sphere = Box::new(Sphere::new(center, radius, mat));
                world.push(sphere);
            } else {
                let mat = Lambertian { albedo };
                let sphere = Box::new(Sphere::new(center, radius, mat));
                world.push(sphere);
            };
        }
    }
    world
}

fn main() -> std::io::Result<()> {
    let world: World = gen_scene(20, 20);

    let image_width = 800;
    let image_height = 600;
    let samples_per_pixel = 32;
    let max_depth = 8;
    let defocus_angle = 0.6;
    let focus_distance = 10.0;

    let camera = Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        focus_distance,
        defocus_angle,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
        20.0,
        0.0..Float::MAX,
    );

    println!("Rendering...");
    let now = Instant::now();
    let image = camera.render(&world);
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
