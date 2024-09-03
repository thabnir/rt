use crate::{
    hittable::{Hit, World},
    intersection::Intersection,
    material::Scatter,
    vec3::{Point3, Ray, Vec3, Vec3Ext},
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::thread_rng;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::Range,
};

pub type Float = f64;

// Min and max distances for rendering
pub const T_MIN: Float = 0.0;
pub const T_MAX: Float = Float::MAX;

#[derive(Default)]
pub struct Camera {
    pub center: Point3,
    pub image_width: usize,
    pub image_height: usize,
    samples_per_pixel: usize,
    max_depth: usize,
    defocus_angle: Float,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    pub pixel00_loc: Vec3,
    pub pixel_du: Vec3,
    pub pixel_dv: Vec3,
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
    // TODO: there's no fucking way mine works right if this is how much they're doing for this
    // reimplementation of pbrt
    // https://github.com/wahn/rs_pbrt/blob/master/src/samplers/halton.rs
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
        center: Vec3,
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
        let w = (center - lookat).normalize();
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

        let vp_upper_left = center - (w * focus_distance) - viewport_u / 2.0 - viewport_v / 2.0;

        // Top left pixel center
        let pixel00_loc = vp_upper_left + (pixel_du + pixel_dv) / 2.0;

        let defocus_radius = focus_distance * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        let rng_map = halton_sequence(2, 1024 * 1024)
            .zip(halton_sequence(3, 1024 * 1024))
            .collect_vec();

        Camera {
            center,
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
        // Halton sequence sampling (I have no idea if I'm doing this right, I think not, but IDK)
        // https://psgraphics.blogspot.com/2018/10/flavors-of-sampling-in-ray-tracing.html
        // TODO: adaptive sampling? ReSTIR? No idea!
        // https://cs184.eecs.berkeley.edu/sp24/docs/hw3-1-part-5
        // https://cseweb.ucsd.edu/classes/sp17/cse168-a/CSE168_07_Random.pdf
        // https://cs184.eecs.berkeley.edu/sp24

        let offset = self.rng_map[i];

        let pixel_sample = self.pixel00_loc
            + (self.pixel_du * (x as Float + offset.0))
            + (self.pixel_dv * (y as Float + offset.1));
        // TODO: make this use an Option<Float> instead of a Float for when I want no blur at all
        // Then it can avoid accessing the rng_map and doing extra math it doesn't have to
        // kind of annoying since it requires some Camera refactoring
        let origin = if self.defocus_angle <= 0.0 {
            self.center // no blur
        } else {
            // TODO: implement better sampling technique for this (QMC stuff)
            self.defocus_disk_sample() // random blur
        };
        Ray::new(origin.into(), pixel_sample - origin)
    }

    pub fn debug_ray(&self, x: f64, y: f64) -> Ray {
        let pixel_sample =
            self.pixel00_loc + (self.pixel_du * (x as Float)) + (self.pixel_dv * (y as Float));
        Ray::new(self.center.into(), pixel_sample - self.center)
    }

    pub fn debug_raycast<'a>(
        &self,
        world: &'a World,
        ray: &Ray,
    ) -> Option<(Intersection<'a>, Vec3, Option<Ray>)> {
        if let Some(hit) = world.hit(ray, &(0.001..self.t_range.end)) {
            if let Some((attenuation, scattered)) = hit.material.scatter(ray, &hit) {
                Some((hit, attenuation, Some(scattered)))
            } else {
                Some((hit, Vec3::zeros(), None)) // Light was absorbed, not scattered
            }
        } else {
            None
        }
    }

    /// Fires a ray from the camera into the world and recursively bounces to determine the ray's color
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
            // Ray missed all other objects and hit the sky box
            let direction = ray.direction.normalize();
            world.sky_color_toward(&direction)
        }
    }

    pub fn render_pixel(&self, world: &World, x: usize, y: usize, num_samples: usize) -> Vec3 {
        (0..num_samples)
            .into_par_iter()
            .map(|i| {
                // TODO: the way this uses its "random" samples is really suspicious...
                let ray = self.get_ray(x, y, i);
                self.raycast(world, &ray, self.max_depth)
            })
            .sum::<Vec3>()
            / num_samples as Float // average color across all samples
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

        // Write the colors in the PPM format with integer RGB values in [0, 255]
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
        // TODO: QMC? No idea how, though!
        let p: Vec3 = Vec3::random_in_unit_disc(&mut thread_rng());
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }
}
