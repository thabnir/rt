use crate::{
    hittable::{Hit, World},
    ray::Ray,
    vec3::{Color, Point3, Vec3},
};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Range,
};

pub type Float = f32;

// Min and max distances for rendering
pub const T_MIN: Float = 0.0;
pub const T_MAX: Float = Float::MAX;

pub struct Camera {
    center: Vec3<Float>,
    image_width: u16,
    image_height: u16,
    samples_per_pixel: u16,
    max_depth: u16,
    defocus_angle: Float,
    defocus_disk_u: Vec3<Float>,
    defocus_disk_v: Vec3<Float>,
    pixel00_loc: Vec3<Float>,
    pixel_du: Vec3<Float>,
    pixel_dv: Vec3<Float>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lookfrom: Vec3<Float>,
        lookat: Vec3<Float>,
        up: Vec3<Float>,
        focus_distance: Float, // Distance from camera's center to plane of perfect focus
        defocus_angle: Float,  // Variation of angle of rays through each pixel
        image_width: u16,
        image_height: u16,
        samples_per_pixel: u16,
        max_depth: u16,
        vertical_fov: Float,
        t_range: Range<Float>,
    ) -> Self {
        let w = (lookfrom - lookat).normalized();
        let u = up.cross(w).normalized();
        let v = w.cross(u);
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

        let defocus_radius = focus_distance * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;
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
        }
    }

    /// Return a camera ray originating from the defocus disk and directed at a random
    /// point around the pixel location `x, y`.
    fn get_ray(&self, x: u16, y: u16) -> Ray<Float> {
        // Offsets uniformly distributed within 1/2 pixel ensure 100% coverage with 0 overlap
        let range = Uniform::from(-0.5..0.5);
        let mut rng = thread_rng();
        let x_offset: Float = range.sample(&mut rng);
        let y_offset: Float = range.sample(&mut rng);
        let pixel_sample = self.pixel00_loc
            + (self.pixel_du * (x as Float + x_offset))
            + (self.pixel_dv * (y as Float + y_offset));
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center // no blur
        } else {
            self.defocus_disk_sample() // random blur
        };
        let ray_dir = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_dir)
    }

    fn raycast(&self, world: &World<Float>, ray: &Ray<Float>, max_depth: u16) -> Color<Float> {
        if let Some(hit) = world.hit(ray, &(0.001..self.t_range.end)) {
            if let Some((attenuation, scattered)) = hit.material.scatter(ray, &hit) {
                // Recursively send out new rays as they bounce until the depth limit
                if max_depth > 0 {
                    attenuation * self.raycast(world, &scattered, max_depth - 1)
                } else {
                    Color::new(1.0, 0.0, 0.0) // Bounce limit reached
                }
            } else {
                Color::new(0.0, 0.0, 0.0) // No ray collision -> void -> return black
                                          // Don't think this should ever actually happen bc skybox
            }
        } else {
            let unit_dir = ray.direction.normalized();
            let a = (unit_dir.y + 1.0) / 2.0;
            Vec3::one() * (1.0 - a) + Vec3::new(0.5, 0.7, 1.0) * a
        }
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
        // Obvious room for optimization here but whatever, it's fast enough as is
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

    /// Returns a random point in the camera's defocus disk
    fn defocus_disk_sample(&self) -> Point3<Float> {
        let p: Vec3<Float> = Vec3::random_in_unit_disc(&mut thread_rng());
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }
}
