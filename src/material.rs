use crate::{
    camera::{Float, Image},
    intersection::Intersection,
    texture::{ImageTexture, SolidColor, Texture, TextureEnum},
    vec3::{Ray, Vec3, Vec3Ext},
};
use enum_dispatch::enum_dispatch;
use rand::{thread_rng, Rng};

#[enum_dispatch]
#[derive(Debug)]
pub enum Material {
    Lambertian,
    Metal,
    Dielectric,
}

impl Material {
    // TODO: figure out materials
    pub fn from_gltf(gltf_mat: gltf::Material, image: Option<Image>) -> Self {
        let pbr = gltf_mat.pbr_metallic_roughness();
        let fuzz = pbr.roughness_factor().into();
        let color = pbr.base_color_factor().map(|x| x.into());

        if let Some(image) = image {
            let tex = ImageTexture::new(image).into();
            return Metal::new(tex, Some(fuzz)).into();
        }

        let color = Vec3::new(color[0], color[1], color[2]);

        Metal::new_solid(color, Some(fuzz)).into()
    }
}
// TODO: change out uses of Vec3 for a Color type where applicable. Make said Color type.
// Make invalid states unrepresentable and whatnot.

#[enum_dispatch(Material)]
pub trait Scatter: Send + Sync {
    // TODO: I don't think this needs to be an option type?
    // At the very least, between Lambertian, Dielectric, and Metal's `Scatter` implementations,
    // there is not one instance in which `None` is returned
    fn scatter(&self, ray_in: &Ray, record: &Intersection) -> Option<(Vec3, Ray)>;
}

fn reflect(incoming_direction: Vec3, surface_normal: Vec3) -> Vec3 {
    // Scale normal by length of incoming ray's direction projected onto the normal
    // Then reflect the ray by subtracting twice its height relative to the surface
    let scaled_normal = surface_normal * incoming_direction.dot(&surface_normal);
    incoming_direction - scaled_normal * 2.0
}

/// Expects `incoming_direction` to be a unit vector
fn refract(incoming_direction: Vec3, surface_normal: Vec3, refractive_ratio: Float) -> Vec3 {
    let cos_theta = (-incoming_direction.dot(&surface_normal)).min(1.0);
    let r_out_perp = (incoming_direction + surface_normal * cos_theta) * refractive_ratio;
    let x = -((1.0 - r_out_perp.norm_squared()).abs().sqrt());
    let r_out_parallel = surface_normal * x;
    r_out_parallel + r_out_perp
}

#[derive(Debug)]
pub struct Lambertian {
    pub texture: TextureEnum,
}

impl Lambertian {
    pub fn new(texture: TextureEnum) -> Self {
        Lambertian { texture }
    }

    pub fn new_rgb_solid(r: Float, g: Float, b: Float) -> Self {
        let texture = SolidColor::new_rgb(r, g, b);
        Lambertian::new(texture.into())
    }
}

#[derive(Debug)]
pub struct Metal {
    pub texture: TextureEnum,
    pub fuzz: Option<Float>,
}

impl Metal {
    pub fn new_solid(color: Vec3, fuzz: Option<Float>) -> Self {
        let solid_texture = SolidColor::new(color).into();
        Metal::new(solid_texture, fuzz)
    }
    pub fn new(texture: TextureEnum, fuzz: Option<Float>) -> Self {
        Metal { texture, fuzz }
    }
}

impl Scatter for Metal {
    fn scatter(&self, ray_in: &Ray, intersection: &Intersection) -> Option<(Vec3, Ray)> {
        let reflected_dir = if let Some(fuzz) = self.fuzz {
            reflect(ray_in.direction, intersection.normal)
                + Vec3::random_unit(&mut thread_rng()) * fuzz
        } else {
            reflect(ray_in.direction, intersection.normal)
        };
        let scattered = Ray::new(intersection.point.into(), reflected_dir);
        let attenuation =
            self.texture
                .value(intersection.uv.x, intersection.uv.y, intersection.point);
        Some((attenuation, scattered))
    }
}

impl Scatter for Lambertian {
    fn scatter(&self, _ray_in: &Ray, hit: &Intersection) -> Option<(Vec3, Ray)> {
        let mut scatter_dir = hit.normal + Vec3::random_unit(&mut thread_rng());
        if scatter_dir.near_zero() {
            scatter_dir = hit.normal;
        }
        let scattered = Ray::new(hit.point.into(), scatter_dir);
        let attenuation = self.texture.value(hit.uv.x, hit.uv.y, hit.point);
        Some((attenuation, scattered))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Dielectric {
    /// Refractive index in vacuum or air, or the ratio of the material's RI over the RI of the enclosing medium
    pub refractive_index: Float,
    /// Controls the amount of "fuzz" on the surface. Higher values make the glass look frosted
    pub fuzz: Option<Float>,
}

impl Dielectric {
    pub fn new(refractive_index: Float) -> Self {
        Dielectric {
            refractive_index,
            fuzz: None,
        }
    }

    pub fn new_frosted(refractive_index: Float, fuzz: Float) -> Self {
        Dielectric {
            refractive_index,
            fuzz: Some(fuzz),
        }
    }

    pub fn new_inside_other(material_index: Float, container_index: Float) -> Self {
        Dielectric::new(material_index / container_index)
    }
}

impl Scatter for Dielectric {
    fn scatter(&self, ray_in: &Ray, record: &Intersection) -> Option<(Vec3, Ray)> {
        let ri = if record.is_front_face {
            1.0 / self.refractive_index
        } else {
            self.refractive_index
        };

        let incoming_direction = ray_in.direction.normalize();

        let cos_theta = (-incoming_direction.dot(&record.normal)).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt(); // sin^2(x) + cos^2(x) = 1
        let cannot_refract = ri * sin_theta > 1.0;

        let noise = thread_rng().gen_range(0.0..=1.0);

        let direction = if cannot_refract || reflectance(cos_theta, ri) > noise {
            reflect(incoming_direction, record.normal)
        } else if let Some(surface_fuzz) = self.fuzz {
            refract(incoming_direction, record.normal, ri)
                + Vec3::random_unit(&mut thread_rng()) * surface_fuzz
        } else {
            refract(incoming_direction, record.normal, ri)
        };
        Some((
            Vec3::ONE,
            Ray::new(record.point.into(), direction.normalize()),
        ))
    }
}

/// Returns Schlick's approximation for reflectance at a given angle.
fn reflectance(cosine: Float, refractive_index: Float) -> Float {
    let r0 = (1.0 - refractive_index) / (1.0 + refractive_index);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
