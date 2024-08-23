use crate::{
    camera::{Camera, Float},
    hittable::{Sphere, World},
    material::{Dielectric, Lambertian, Metal},
    texture::{CheckerTexture, ImageTexture, SolidColor},
    vec3::{Vec3, Vec3Ext},
    window::{HEIGHT, WIDTH},
};
use image::RgbImage;
use rand::{thread_rng, Rng};
use std::io;

pub fn cam1() -> Camera {
    let image_width = WIDTH as usize;
    let image_height = HEIGHT as usize;
    let samples_per_pixel = 32; // not relevant for window_preview
    let max_depth = 100;
    let defocus_angle = 1.0;
    let focus_distance = 10.0;

    Camera::new(
        Vec3::new(12.0, 2.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        focus_distance,
        defocus_angle,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
        20.0,
        0.0..Float::MAX,
    )
}

pub fn cam2() -> Camera {
    let image_width = WIDTH as usize;
    let image_height = HEIGHT as usize;
    let samples_per_pixel = 32; // not relevant for window_preview
    let max_depth = 100;
    let defocus_angle = 0.7;
    let focus_distance = 16.0;

    let lookfrom = Vec3::new(14.0, 3.0, 10.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);

    Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 0.0, 1.0),
        focus_distance,
        defocus_angle,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
        20.0,
        0.0..Float::MAX,
    )
}

pub fn topdown_cam() -> Camera {
    let image_width = WIDTH as usize;
    let image_height = HEIGHT as usize;
    let samples_per_pixel = 32; // not relevant for window_preview
    let max_depth = 100;
    let defocus_angle = 0.7;

    let up = Vec3::new(0.0, 0.0, 1.0); // let Z be the up direction

    // TODO: figure out why it breaks when lookfrom is along the z axis
    let lookfrom = Vec3::new(0.1, 0.1, 20.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);

    let focus_distance = lookfrom.metric_distance(&lookat);

    Camera::new(
        lookfrom,
        lookat,
        up,
        focus_distance,
        defocus_angle,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
        20.0,
        0.0..Float::MAX,
    )
}

pub fn earth_scene() -> io::Result<World> {
    let mut shapes = Vec::new();
    let earth_bytes: &[u8] = include_bytes!("./assets/textures/earth.png");
    let earth_image: RgbImage = ImageTexture::load_embedded_image(earth_bytes);
    let earth_tex = ImageTexture::new(earth_image).into();
    let earth_mat = Lambertian::new(earth_tex).into();
    let earth_ball = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0, earth_mat).into();

    shapes.push(earth_ball);

    Ok(World::build(shapes))
}

// TODO: figure out what the fuck is up with this weird moiré pattern looking abomination
// NOTES: it only appears all fucked up like that using my "Halton Sampling" (but still shows up
// minus the weird patterns when using thread_rng())
// (scare quotes placed intentionally, that shit is NOT how you're supposed to do it)
// (very unsure as to why it's normally indistinguishable anyhow)
// (should probably un-implement it until i've actually figured out how the fuck it works)
pub fn cover_scene(grid_i: i16, grid_j: i16, camera: &Camera) -> World {
    let mut rng = thread_rng();
    let mut shapes = Vec::new();

    let earth_bytes = include_bytes!("./assets/textures/earth.png");
    let mars_bytes = include_bytes!("./assets/textures/mars.jpg");
    let thing_bytes = include_bytes!("./assets/textures/moon_hires.jpg");
    let saul_bytes = include_bytes!("./assets/textures/saul.webp");

    let earth_image = ImageTexture::load_embedded_image(earth_bytes);
    let mars_image = ImageTexture::load_embedded_image(mars_bytes);
    let thing_image = ImageTexture::load_embedded_image(thing_bytes);
    let saul_image = ImageTexture::load_embedded_image(saul_bytes);

    let earth_tex = ImageTexture::new(earth_image).into();
    let mars_tex = ImageTexture::new(mars_image).into();
    let moon_tex = ImageTexture::new(thing_image).into();
    let saul_tex = ImageTexture::new(saul_image).into();

    let even_texture = SolidColor::new(Vec3::new(0.0, 0.0, 0.0)).into();
    let odd_texture = SolidColor::new(Vec3::new(0.95, 0.95, 0.95)).into();
    let checker_tex = CheckerTexture::new(0.31, even_texture, odd_texture).into();
    let checker_mat = Lambertian::new(checker_tex).into();

    let mars_mat = Lambertian::new(mars_tex).into();
    let earth_mat = Lambertian::new(earth_tex).into();
    let moon_mat = Lambertian::new(moon_tex).into();
    let saul_mat = Lambertian::new(saul_tex).into();
    let mat1 = Dielectric::new(1.5).into();
    let mat3 = Metal::new_solid(Vec3::new(0.7, 0.6, 0.5), None).into();

    let ground = Vec3::new(0.0, 0.0, -1000.0);
    let big_6_radius = 0.7;

    let saul_loc = Vec3::new(-1.0, 1.732, big_6_radius); // Top-left sphere
    let p1 = Vec3::new(-1.0, -1.732, big_6_radius); // Bottom-left sphere
    let p2 = Vec3::new(2.0, 0.0, big_6_radius); // Right sphere
    let p3 = Vec3::new(-2.0, 0.0, big_6_radius); // Left sphere
    let p4 = Vec3::new(1.0, 1.732, big_6_radius + 0.5); // Top-right sphere
    let p5 = Vec3::new(1.0, -1.732, big_6_radius); // Bottom-right sphere

    shapes.push(Sphere::new(ground, ground.z.abs(), checker_mat).into());
    shapes.push(Sphere::new(p1, big_6_radius, mat1).into());
    shapes.push(Sphere::new(p2, big_6_radius, mars_mat).into());
    shapes.push(Sphere::new(p3, big_6_radius, mat3).into());
    shapes.push(Sphere::new(p4, big_6_radius, earth_mat).into());
    shapes.push(Sphere::new(p5, big_6_radius, moon_mat).into());

    let you_the_viewer = camera.center;
    let saul_sphere = Sphere::new_facing(saul_loc, big_6_radius, saul_mat, you_the_viewer).into();
    shapes.push(saul_sphere);

    for i in -grid_i..grid_i {
        for j in -grid_j..grid_j {
            let radius = 0.2;
            let albedo: Vec3 = Vec3::random(&mut rng, 0.0, 1.0);
            let offset: Vec3 = Vec3::new(rng.gen_range(0.0..0.9), rng.gen_range(0.0..0.9), 0.0);
            let i_offset = 1.0;
            let j_offset = 1.0;
            let center = Vec3::new(i as Float * i_offset, j as Float * j_offset, radius) + offset;

            // Don't put it too close to the big boys
            let collide_dist = radius + big_6_radius;
            if center.metric_distance(&p1) < collide_dist
                || center.metric_distance(&p2) < collide_dist
                || center.metric_distance(&p3) < collide_dist
                || center.metric_distance(&p4) < collide_dist
                || center.metric_distance(&saul_loc) < collide_dist
                || center.metric_distance(&p5) < collide_dist
            {
                continue;
            }
            let choose = rng.gen_range(0.0..1.0);
            let sphere = {
                let mat = {
                    if choose > (0.95) {
                        Dielectric::new(1.5).into()
                    } else if choose > 0.8 {
                        let fuzz = rng.gen_range(0.0..0.5);
                        Metal::new_solid(albedo, Some(fuzz)).into()
                    } else {
                        let texture = SolidColor::new(albedo).into();
                        Lambertian::new(texture).into()
                    }
                };
                Sphere::new(center, radius, mat).into()
            };

            shapes.push(sphere);
        }
    }
    World::build(shapes)
}

pub fn gen_checkered() -> World {
    let mut shapes = Vec::new();

    let even_texture = SolidColor::new(Vec3::new(0.2, 0.3, 0.1)).into();
    let odd_texture = SolidColor::new(Vec3::new(0.9, 0.9, 0.9)).into();

    let checker_tex = CheckerTexture::new(0.31, even_texture, odd_texture).into();

    let mat1 = Lambertian::new(checker_tex).into();

    let even_texture = SolidColor::new(Vec3::new(0.2, 0.3, 0.1)).into();
    let odd_texture = SolidColor::new(Vec3::new(0.9, 0.9, 0.9)).into();
    let checker_tex = CheckerTexture::new(0.31, even_texture, odd_texture).into();
    let mat2 = Lambertian::new(checker_tex).into();

    let sphere_lower = Sphere::new(Vec3::new(0.0, -10.0, 0.0), 10.0, mat1).into();
    let sphere_upper = Sphere::new(Vec3::new(0.0, 10.0, 0.0), 10.0, mat2).into();
    shapes.push(sphere_lower);
    shapes.push(sphere_upper);
    World::build(shapes)
}
