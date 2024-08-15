use std::sync::Arc;

use crate::{
    camera::Float,
    vec3::{Point3, Vec3},
};
use image::{load_from_memory, RgbImage};

pub trait Texture {
    fn value(&self, u: Float, v: Float, point: Point3) -> Vec3;
}

pub enum TextureEnum {
    SolidColor(SolidColor),
    CheckerTexture(CheckerTexture),
    ImageTexture(ImageTexture),
}

impl Texture for TextureEnum {
    fn value(&self, u: Float, v: Float, point: Point3) -> Vec3 {
        match self {
            TextureEnum::SolidColor(t) => t.value(u, v, point),
            TextureEnum::CheckerTexture(t) => t.value(u, v, point),
            TextureEnum::ImageTexture(t) => t.value(u, v, point),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SolidColor {
    pub color: Vec3,
}

impl Texture for SolidColor {
    fn value(&self, _u: Float, _v: Float, _point: Point3) -> Vec3 {
        self.color
    }
}

impl SolidColor {
    pub fn new(color: Vec3) -> Self {
        Self { color }
    }

    pub fn new_rgb(r: Float, g: Float, b: Float) -> Self {
        Self {
            color: Vec3::new(r, g, b),
        }
    }
}

pub struct CheckerTexture {
    /// Larger scale values correspond to larger checker sizes
    scale_inverted: Float,
    even_texture: Arc<TextureEnum>, // Boxed to avoid infinite size with recursion
    odd_texture: Arc<TextureEnum>,
}

impl CheckerTexture {
    pub fn new(
        scale: Float,
        even_texture: Arc<TextureEnum>,
        odd_texture: Arc<TextureEnum>,
    ) -> Self {
        CheckerTexture {
            scale_inverted: 1.0 / scale,
            even_texture,
            odd_texture,
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: Float, v: Float, point: Point3) -> Vec3 {
        let x_int = (self.scale_inverted * point.x).floor() as i32;
        let y_int = (self.scale_inverted * point.y).floor() as i32;
        let z_int = (self.scale_inverted * point.z).floor() as i32;

        let is_even = (x_int + y_int + z_int) % 2 == 0;
        if is_even {
            self.even_texture.value(u, v, point)
        } else {
            self.odd_texture.value(u, v, point)
        }
    }
}

pub struct ImageTexture {
    pub image: Arc<RgbImage>,
}

impl ImageTexture {
    pub fn load_embedded_image(data: &[u8]) -> RgbImage {
        load_from_memory(data)
            .expect("Failed to load image")
            .to_rgb8()
    }

    pub fn new(image: RgbImage) -> Self {
        ImageTexture {
            image: Arc::new(image),
        }
    }
    // pub fn new_from_embedded_texture(data: &[u8]) -> Self {
    //     let img = load_from_memory(data)
    //         .expect("Failed to load image")
    //         .to_rgb8();
    //     ImageTexture { image: img }
    // }
    //
    // pub fn new_from_path(filepath: &str) -> Self {
    //     // TODO: figure out how to lay the textures out so this works
    //     let img = image::open(filepath)
    //         .unwrap_or_else(|_| panic!("Failed to load image from path `{}`", filepath))
    //         .to_rgb8();
    //     ImageTexture { image: img }
    // }
}

impl Texture for ImageTexture {
    fn value(&self, u: Float, v: Float, _point: Point3) -> Vec3 {
        let r = 0.0..1.0;
        if self.image.height() == 0 || self.image.width() == 0 || !r.contains(&u) || !r.contains(&v)
        {
            return Vec3::new(1.0, 0.0, 0.0); // Debug color
        }

        // Clamp input coords to [0, 1]
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // Flip `v` to image coordinates

        let i = (u * (self.image.width() - 1) as Float) as u32;
        let j = (v * (self.image.height() - 1) as Float) as u32;

        let pixel = self.image[(i, j)];
        let [r, g, b] = pixel.0;
        // TODO: vec3 from 8-bit rgb color function
        let scale = 1.0 / 255.0;
        Vec3::new(r as Float, g as Float, b as Float) * scale
    }
}
