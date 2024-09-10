#![allow(unused)]
use std::sync::Arc;

use scenes::sponza;

use crate::{
    hittable::World,
    material::Lambertian,
    material::{Dielectric, Material, Metal},
    texture::{CheckerTexture, SolidColor},
    vec3::Vec3,
};

pub mod camera;
pub mod hittable;
pub mod intersection;
pub mod material;
pub mod scenes;
pub mod texture;
pub mod vec3;
pub mod window;

fn main() {
    env_logger::init();
    std::env::set_var("RUST_BACKTRACE", "FULL");

    let camera = scenes::cam1();

    let mut shapes = Vec::new();

    let even_texture = SolidColor::new(Vec3::new(0.1, 0.1, 0.1)).into();
    let odd_texture = SolidColor::new(Vec3::new(0.95, 0.95, 0.95)).into();
    let checker_tex = CheckerTexture::new(3.0, even_texture, odd_texture).into();
    let checker_mat: Arc<Material> = Arc::new(Lambertian::new(checker_tex).into());
    let frosty_glass: Arc<Material> = Arc::new(Dielectric::new_frosted(1.5, 0.05).into());

    let plaster: Arc<Material> = Arc::new(Lambertian::new_rgb_solid(1.0, 1.0, 1.0).into());

    let ground_height = -0.2;
    let mut ground = scenes::generate_ground_plane(
        10000.0,
        10000.0,
        ground_height,
        checker_mat.clone(),
        // frosty_glass.clone(),
        true,
    );

    shapes.append(&mut ground);
    // shapes.append(&mut scenes::triangle_scene());
    // shapes.append(&mut scenes::mesh_scene());
    shapes.append(&mut scenes::cover_scene(300, 300, &camera, ground_height));
    // shapes.append(&mut scenes::triangle_scene());
    shapes.append(&mut scenes::gltf_test());
    // shapes.append(&mut sponza());
    println!("Rendering a scene with {} shapes", shapes.len());
    let world = World::build(shapes);

    if let Err(err) = window::render_with_preview(camera, world) {
        println!("Err: {}", err);
    }
}
