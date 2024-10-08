use crate::{
    camera::{Float, Image},
    vec3::{Point3, Vec3},
};
use enum_dispatch::enum_dispatch;

#[enum_dispatch(TextureEnum)]
pub trait Texture {
    fn value(&self, u: Float, v: Float, point: Point3) -> Vec3;
}

#[enum_dispatch]
#[derive(Debug)]
pub enum TextureEnum {
    SolidColor,
    CheckerTexture,
    ImageTexture,
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

#[derive(Debug)]
pub struct CheckerTexture {
    /// Larger scale values correspond to larger checker sizes
    scale_inverted: Float,
    even_texture: Box<TextureEnum>, // Boxed to avoid infinite size with recursion
    odd_texture: Box<TextureEnum>,
}

impl CheckerTexture {
    pub fn new(scale: Float, even_texture: TextureEnum, odd_texture: TextureEnum) -> Self {
        CheckerTexture {
            scale_inverted: 1.0 / scale,
            even_texture: Box::new(even_texture),
            odd_texture: Box::new(odd_texture),
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
    pub image: Image,
}

impl std::fmt::Debug for ImageTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageTexture")
            .field("image", &"<image data>")
            .finish()
    }
}

impl ImageTexture {
    pub fn load_embedded_image(data: &[u8]) -> Image {
        let img = image::load_from_memory(data).expect("Failed to load image");
        img.into()
    }

    pub fn new(image: Image) -> Self {
        ImageTexture { image }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: Float, v: Float, _point: Point3) -> Vec3 {
        // let r = 0.0..=1.0;
        // if self.image.height == 0 || self.image.width == 0 || !r.contains(&u) || !r.contains(&v) {
        // println!("Error: (u, v)=({}, {}) out of bounds", u, v);
        // return Vec3::new(1.0, 0.0, 0.0); // Debug color
        // }

        // Clamp input coords to [0, 1]
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let x = (u * (self.image.width - 1) as Float) as usize;
        let y = (v * (self.image.height - 1) as Float) as usize;

        // Debug view:
        // Vec3::new(u, v, (1.0 - u - v).max(0.0))

        self.image[(x, y)]
    }
}
