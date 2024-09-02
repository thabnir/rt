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
    std::env::set_var("RUST_BACKTRACE", "1");
    let camera = scenes::widecam();
    // let world = scenes::cover_scene(60, 60, &camera);
    let world = scenes::mesh_scene();

    if let Err(err) = window::render_with_preview(camera, world) {
        println!("Err: {}", err);
    }
}
