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
    let camera = scenes::cam2();
    let world = scenes::cover_scene(60, 60, &camera);

    if let Err(err) = window::render_with_preview(camera, world) {
        println!("Err: {}", err);
    }
}
