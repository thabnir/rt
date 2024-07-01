use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Range,
};

use crate::{
    hittable::{Hittable, World},
    ray::Ray,
    vec3::{Color, Vec3},
};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use rayon::prelude::*;

pub type Float = f32;
pub type UInt = u16;

pub const T_MIN: Float = 0.0; // maybe 0
pub const T_MAX: Float = Float::MAX;

pub struct Camera {
    center: Vec3<Float>,
    // focal_length: Float,
    image_width: u16,
    image_height: u16,
    samples_per_pixel: u16,
    max_depth: u16,
    // viewport_width: Float,
    // viewport_height: Float,
    pixel00_loc: Vec3<Float>,
    pixel_dx: Vec3<Float>,
    pixel_dy: Vec3<Float>,
    t_range: Range<Float>,
}

pub type Pixel = (u16, u16, Color<Float>);
pub struct Image {
    colors: Vec<Pixel>,
    width: u16,
    height: u16,
}

/// Take a positive color value in linear space from 0.0 to 1.0 and convert it to gamma 2
pub fn linear_to_gamma<Scalar: num_traits::Float>(linear_color_value: Scalar) -> Scalar {
    linear_color_value.sqrt()
}

impl Camera {
    pub fn new(
        center: Vec3<Float>,
        focal_length: Float,
        image_width: u16,
        image_height: u16,
        samples_per_pixel: u16,
        max_depth: u16,
        viewport_height: Float,
        t_range: Range<Float>,
    ) -> Self {
        let aspect_ratio = image_width as Float / image_height as Float;
        let viewport_width = viewport_height * aspect_ratio;
        // Displacement vectors from left to right and top to bottom of viewport
        let viewport_x = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_y = Vec3::new(0.0, -viewport_height, 0.0);

        // Viewport distance between pixels
        let pixel_dx = viewport_x / (image_width as Float);
        let pixel_dy = viewport_y / (image_height as Float);

        let vp_upper_left =
            center - Vec3::new(0.0, 0.0, focal_length) - viewport_x / 2.0 - viewport_y / 2.0;

        let pixel00_loc = vp_upper_left + (pixel_dx + pixel_dy) / 2.0;
        Camera {
            center,
            // focal_length,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            // viewport_width,
            // viewport_height,
            pixel00_loc,
            pixel_dx,
            pixel_dy,
            t_range,
        }
    }

    fn get_ray(&self, x: u16, y: u16) -> Ray<Float> {
        // Offsets uniformly distributed within 1/2 pixel ensure 100% coverage with 0 overlap
        let range = Uniform::from(-0.5..0.5);
        let mut rng = thread_rng();
        let x_offset: Float = range.sample(&mut rng);
        let y_offset: Float = range.sample(&mut rng);
        let pixel_sample = self.pixel00_loc
            + (self.pixel_dx * (x as Float + x_offset))
            + (self.pixel_dy * (y as Float + y_offset));
        let ray_dir = pixel_sample - self.center;
        Ray::new(self.center, ray_dir)
    }

    fn raycast(&self, world: &World<Float>, ray: &Ray<Float>, max_depth: u16) -> Color<Float> {
        if let Some(hit) = world.hit(ray, &(0.001..self.t_range.end)) {
            // let bounce_dir = Sphere::random_on_hemisphere(&hit.normal); // even light diffusion
            let bounce_dir = hit.normal + Vec3::random_unit(&mut thread_rng()); // lambertian light diffusion (better)
            let bounce_ray = Ray::new(hit.point, bounce_dir);

            // recursively send out new rays as they bounce until the depth limit
            if max_depth > 0 {
                let bounce = self.raycast(world, &bounce_ray, max_depth - 1);
                return bounce * 0.2; // return half the bounced color
            }
            return Color::new(0.0, 0.0, 0.0);
        }
        let unit_dir = ray.direction.normalized();
        let a = (unit_dir.y + 1.0) / 2.0;
        Vec3::one() * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
    }

    pub fn render(&self, world: &World<Float>, progress_bar: ProgressBar) -> Image {
        let colors = (0..self.image_height)
            .into_par_iter()
            .progress_with(progress_bar)
            .flat_map(|y| {
                (0..self.image_width).into_par_iter().map(move |x| {
                    let pixel_color = (0..self.samples_per_pixel)
                        .into_par_iter()
                        .map(|_| {
                            let ray = self.get_ray(x, y);
                            self.raycast(world, &ray, self.max_depth)
                        })
                        .sum::<Color<Float>>()
                        / self.samples_per_pixel as Float; // average color across all samples
                    (x, y, pixel_color)
                })
            })
            .collect();

        Image {
            colors,
            width: self.image_width,
            height: self.image_height,
        }
    }

    pub fn write_image(
        image: Image,
        out_file: File,
        progress_bar: Option<ProgressBar>,
    ) -> std::io::Result<()> {
        let mut buf_writer = BufWriter::new(out_file);

        // Write header metadata necessary for PPM file:
        let header = format!(
            "P3\n{} {} # width, height\n255 # max color value\n",
            image.width, image.height
        );
        buf_writer.write_all(header.as_bytes())?;

        // Write the colors to the buffer
        // Obvious room for optimization here but whatever
        // (e.g. make a big string in advance before writing)
        for (x, _y, color) in image.colors {
            buf_writer.write_all(color.as_rgb().as_bytes())?;
            if let Some(pb) = &progress_bar {
                pb.inc(1);
            }
            if x == image.width - 1 {
                buf_writer.write_all("\n".as_bytes())?;
            } else {
                buf_writer.write_all(" ".as_bytes())?;
            }
        }
        buf_writer.flush()?;
        if let Some(pb) = &progress_bar {
            pb.finish();
        }
        Ok(())
    }
}
