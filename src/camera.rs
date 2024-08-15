use crate::{
    hittable::{Hit, Shape, Sphere, World},
    material::{Dielectric, Lambertian, Material, Metal, Scatter},
    texture::{CheckerTexture, ImageTexture, SolidColor, TextureEnum},
    vec3::{Point3, Ray, Vec3, Vec3Ext},
};
use image::RgbImage;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    ops::Range,
    sync::Arc,
};

pub type Float = f32;

// Min and max distances for rendering
pub const T_MIN: Float = 0.0;
pub const T_MAX: Float = Float::MAX;

#[derive(Default)]
pub struct Camera {
    center: Point3,
    pub image_width: usize,
    pub image_height: usize,
    samples_per_pixel: usize,
    max_depth: usize,
    defocus_angle: Float,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    pixel00_loc: Vec3,
    pixel_du: Vec3,
    pixel_dv: Vec3,
    t_range: Range<Float>,
    rng_map: Vec<(Float, Float)>, // Defines the "random" sequnece for pixel samples. Halton sequence for now
}

pub type Pixel = (usize, usize, Vec3);

#[derive(Default)]
pub struct Image {
    pub colors: Vec<Pixel>,
    pub width: usize,
    pub height: usize,
}

// Used to generate pixel sample offset values for rays for faster convergence / less noise
// Maybe use a uniform pattern instead? Need to do more research into this...
// https://en.wikipedia.org/wiki/Halton_sequence
fn halton_sequence(base: u64, sequence_length: u64) -> impl std::iter::Iterator<Item = Float> {
    let mut n = 0;
    let mut d = 1;
    let mut index = 0;
    std::iter::from_fn(move || {
        if index >= sequence_length {
            return None;
        }
        let x = d - n;
        if x == 1 {
            n = 1;
            d *= base;
        } else {
            let mut y = d / base;
            while x < y {
                y /= base;
            }
            n = (base + 1) * y - x;
        }
        index += 1;
        Some(n as Float / d as Float)
    })
}

impl Camera {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lookfrom: Vec3,
        lookat: Vec3,
        up: Vec3,
        focus_distance: Float, // Distance from camera's center to plane of perfect focus
        defocus_angle: Float,  // Variation of angle of rays through each pixel
        image_width: usize,
        image_height: usize,
        samples_per_pixel: usize,
        max_depth: usize,
        vertical_fov: Float,
        t_range: Range<Float>,
    ) -> Self {
        let w = (lookfrom - lookat).normalize();
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);
        let h = (vertical_fov.to_radians() / 2.0).tan();
        let viewport_height = 2.0 * h * focus_distance;

        // Viewport distance between pixels
        let aspect_ratio = image_width as Float / image_height as Float;
        let viewport_width = viewport_height * aspect_ratio;

        // Displacement vectors from left to right and top to bottom of viewport
        let viewport_u = u * viewport_width; // Left to right across horizontal edge
        let viewport_v = -v * viewport_height; // Down vertical edge

        let pixel_du = viewport_u / (image_width as Float);
        let pixel_dv = viewport_v / (image_height as Float);

        let vp_upper_left = lookfrom - (w * focus_distance) - viewport_u / 2.0 - viewport_v / 2.0;

        // Top left pixel center
        let pixel00_loc = vp_upper_left + (pixel_du + pixel_dv) / 2.0;

        // if let Some(da) = defocus_angle {
        let defocus_radius = focus_distance * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;
        // }

        let rng_map = halton_sequence(2, 1024 * 1024)
            .zip(halton_sequence(3, 1024 * 1024))
            .collect_vec();

        Camera {
            center: lookfrom,
            defocus_angle,
            defocus_disk_u,
            defocus_disk_v,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            pixel00_loc,
            pixel_du,
            pixel_dv,
            t_range,
            rng_map,
        }
    }

    /// Return a camera ray originating from the defocus disk and directed at a random
    /// point around the pixel location `x, y`.
    fn get_ray(&self, x: usize, y: usize, i: usize) -> Ray {
        // Halton sequence sampling (I have no idea if I'm doing this right)
        // httpshttps://psgraphics.blogspot.com/2018/10/flavors-of-sampling-in-ray-tracing.html
        // TODO: test if this actually reduces mean error at different sample levels
        // probably just take the pixel MSE with a fixed render with a shitload of samples
        // find a way to graph it
        // https://cseweb.ucsd.edu/classes/sp17/cse168-a/CSE168_07_Random.pdf
        // Also todo: benchmarking performance. less important here but still important

        // TODO: adaptive sampling?
        // https://cs184.eecs.berkeley.edu/sp24/docs/hw3-1-part-5
        // https://cseweb.ucsd.edu/classes/sp17/cse168-a/CSE168_07_Random.pdf
        // https://cs184.eecs.berkeley.edu/sp24
        let offset = self.rng_map[i];

        let pixel_sample = self.pixel00_loc
            + (self.pixel_du * (x as Float + offset.0))
            + (self.pixel_dv * (y as Float + offset.1));
        // let origin = self
        //     .defocus_angle
        //     .map_or(self.center, |_| self.defocus_disk_sample());
        let origin = if self.defocus_angle <= 0.0 {
            self.center // no blur
        } else {
            // TODO: implement better sampling technique for this (QMC stuff)
            self.defocus_disk_sample() // random blur
        };
        Ray::new(origin.into(), pixel_sample - origin)
    }

    fn raycast(&self, world: &World, ray: &Ray, max_depth: usize) -> Vec3 {
        if let Some(hit) = world.hit(ray, &(0.001..self.t_range.end)) {
            if let Some((attenuation, scattered)) = hit.material.scatter(ray, &hit) {
                // Recursively send out new rays as they bounce until the depth limit
                if max_depth > 0 {
                    let bounced_ray = self.raycast(world, &scattered, max_depth - 1);
                    attenuation.component_mul(&bounced_ray)
                } else {
                    Vec3::new(0.0, 0.0, 0.0) // Bounce limit reached
                }
            } else {
                Vec3::new(0.0, 0.0, 0.0) // Light was absorbed, not scattered
            }
        } else {
            // Skybox
            let unit_dir = ray.direction.normalize();
            let a = (unit_dir.y + 1.0) / 2.0;
            Vec3::ONE * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
        }
    }

    pub fn render_pixel(&self, world: &World, x: usize, y: usize, num_samples: usize) -> Vec3 {
        (0..num_samples)
            .into_par_iter()
            .map(|i| {
                let ray = self.get_ray(x, y, i);
                self.raycast(world, &ray, self.max_depth)
            })
            .sum::<Vec3>()
            / num_samples as Float // average color across all samples
    }

    pub fn render_tile(
        &self,
        world: &World,
        tile_x: usize,
        tile_y: usize,
        tile_width: usize,
        tile_height: usize,
    ) -> Vec<(usize, usize, Vec3)> {
        (tile_y..tile_y + tile_height)
            .cartesian_product(tile_x..tile_x + tile_width)
            .collect_vec()
            .into_par_iter()
            .map(|(y, x)| {
                let pixel_color = self.render_pixel(world, x, y, self.samples_per_pixel);
                (x, y, pixel_color)
            })
            .collect()
    }

    pub fn render_image(&self, world: &World) -> Image {
        let colors = (0..self.image_height)
            .cartesian_product(0..self.image_width)
            .collect_vec()
            .into_par_iter()
            .progress()
            .map(|(y, x)| (x, y, self.render_pixel(world, x, y, self.samples_per_pixel)))
            .collect::<_>();

        Image {
            colors,
            width: self.image_width,
            height: self.image_height,
        }
    }

    pub fn write_image(image: Image, out_file: File) -> std::io::Result<()> {
        let mut buf_writer = BufWriter::new(out_file);

        // Write header metadata necessary for PPM file:
        let header = format!(
            "P3\n{} {} # width, height\n255 # max color value\n",
            image.width, image.height
        );
        buf_writer.write_all(header.as_bytes())?;

        // Write the colors to the buffer
        for (x, _y, color) in image.colors.into_iter().progress() {
            buf_writer.write_all(color.as_rgb_gamma_string().as_bytes())?;
            if x == image.width - 1 {
                buf_writer.write_all("\n".as_bytes())?;
            } else {
                buf_writer.write_all(" ".as_bytes())?;
            }
        }
        buf_writer.flush()?;
        Ok(())
    }

    /// Returns a random point in the camera's defocus disk
    fn defocus_disk_sample(&self) -> Vec3 {
        let p: Vec3 = Vec3::random_in_unit_disc(&mut thread_rng());
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }
}

pub fn gen_checkered() -> World {
    let mut shapes = Vec::new();

    let even_texture = Arc::new(TextureEnum::SolidColor(SolidColor::new(Vec3::new(
        0.2, 0.3, 0.1,
    ))));
    let odd_texture = Arc::new(TextureEnum::SolidColor(SolidColor::new(Vec3::new(
        0.9, 0.9, 0.9,
    ))));

    let checker_tex = Arc::new(TextureEnum::CheckerTexture(CheckerTexture::new(
        0.31,
        even_texture,
        odd_texture,
    )));

    let mat1 = Material::Lambertian(Lambertian::new(checker_tex.clone()));
    let mat2 = Material::Lambertian(Lambertian::new(checker_tex.clone()));

    let sphere_lower = Shape::Sphere(Sphere::new(Vec3::new(0.0, -10.0, 0.0), 10.0, mat1));
    let sphere_upper = Shape::Sphere(Sphere::new(Vec3::new(0.0, 10.0, 0.0), 10.0, mat2));
    shapes.push(sphere_lower);
    shapes.push(sphere_upper);
    World::build(shapes)
}

pub fn gen_earth() -> io::Result<World> {
    let mut shapes = Vec::new();
    let earth_bytes: &[u8] = include_bytes!("./assets/textures/earthmap.jpg");
    let earth_image: RgbImage = ImageTexture::load_embedded_image(earth_bytes);
    let earth_tex = ImageTexture::new(earth_image);
    let earth_mat = Material::Lambertian({
        let texture = Arc::new(TextureEnum::ImageTexture(earth_tex));
        Lambertian::new(texture)
    });
    let earth_ball = Shape::Sphere(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0, earth_mat));

    shapes.push(earth_ball);

    Ok(World::build(shapes))
}

pub fn gen_scene(grid_i: i16, grid_j: i16) -> World {
    let mut rng = thread_rng();
    let mut shapes = Vec::new();

    let earth_bytes = include_bytes!("./assets/textures/earthmap.jpg");
    let mars_bytes = include_bytes!("./assets/textures/saul.webp");

    let earth_image = ImageTexture::load_embedded_image(earth_bytes);
    let mars_image = ImageTexture::load_embedded_image(mars_bytes);

    let earth_tex = Arc::new(TextureEnum::ImageTexture(ImageTexture::new(earth_image)));
    let mars_tex = Arc::new(TextureEnum::ImageTexture(ImageTexture::new(mars_image)));

    // let even_texture = TextureEnum::SolidColor(SolidColor::new(Vec3::new(0.0, 0.0, 0.0)));
    // let odd_texture = TextureEnum::SolidColor(SolidColor::new(Vec3::new(0.95, 0.95, 0.95)));
    let checker_tex = CheckerTexture::new(0.31, earth_tex.clone(), mars_tex.clone());
    let checker_tex = TextureEnum::CheckerTexture(checker_tex);
    let checker_tex = Arc::new(checker_tex);

    let ground = Shape::Sphere(Sphere::new(
        Vec3::new(0.0, -1000.0, -1.0),
        1000.0,
        Material::Lambertian({
            let texture = &checker_tex;
            Lambertian {
                texture: texture.clone(),
            }
        }),
    ));
    shapes.push(ground);
    let mat1 = Material::Dielectric(Dielectric {
        refractive_index: 1.5,
    });
    let p1 = Vec3::new(0.0, 1.0, 0.0);
    shapes.push(Shape::Sphere(Sphere::new(p1, 1.0, mat1)));

    // let tex2 = Arc::new(TextureEnum::SolidColor(SolidColor::new(Vec3::new(
    //     0.4, 0.2, 0.1,
    // ))));
    // let mat2 = Material::Lambertian(Lambertian::new(tex2.clone()));

    let mars_mat = Material::Lambertian(Lambertian::new(mars_tex.clone()));
    let p2 = Vec3::new(-3.0, 1.0, 0.0);
    shapes.push(Shape::Sphere(Sphere::new(p2, 1.0, mars_mat)));
    let mat3 = Material::Metal(Metal {
        albedo: Vec3::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });
    let p3 = Vec3::new(-5.0, 1.0, 0.0);
    shapes.push(Shape::Sphere(Sphere::new(p3, 1.0, mat3)));

    let p4 = Vec3::new(3.0, 1.0, 0.0);
    let earth_mat = Material::Lambertian(Lambertian::new(earth_tex.clone()));
    let earth_ball = Shape::Sphere(Sphere::new(p4, 1.0, earth_mat));
    shapes.push(earth_ball);

    for i in -grid_i..grid_i {
        for j in -grid_j..grid_j {
            let radius = 0.2;
            let albedo: Vec3 = Vec3::random(&mut rng, 0.0, 1.0);
            let offset: Vec3 = Vec3::new(rng.gen_range(0.0..0.9), 0.0, rng.gen_range(0.0..0.9));
            let i_offset = 1.0;
            let j_offset = 1.0;
            let center = Vec3::new(i as Float * i_offset, radius, j as Float * j_offset) + offset;

            if center.metric_distance(&p1) < 1.2
                || center.metric_distance(&p2) < 1.2
                || center.metric_distance(&p3) < 1.2
            {
                continue;
            }
            let choose = rng.gen_range(0.0..1.0);
            let sphere = {
                let mat: Material = {
                    if choose > (0.95) {
                        Material::Dielectric(Dielectric {
                            refractive_index: 1.5,
                        })
                    } else if choose > 0.8 {
                        let fuzz = rng.gen_range(0.0..0.5);
                        Material::Metal(Metal { albedo, fuzz })
                    } else {
                        Material::Lambertian({
                            let texture = TextureEnum::SolidColor(SolidColor::new(albedo));
                            Lambertian::new_take(texture)
                        })
                    }
                };
                Shape::Sphere(Sphere::new(center, radius, mat))
            };

            shapes.push(sphere);
        }
    }
    World::build(shapes)
}
