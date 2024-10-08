use crate::solid_texture::SolidTexture;
use crate::texture::Texture;
use crate::vector_3::Vec3;
use std::sync::Arc;

pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture + Sync + Send>,
    odd: Arc<dyn Texture + Sync + Send>,
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, point: Vec3) -> Vec3 {
        let x = (self.inv_scale * point.x).floor() as i32;
        let y = (self.inv_scale * point.y).floor() as i32;
        let z = (self.inv_scale * point.z).floor() as i32;

        let even = (x + y + z) % 2 == 0;
        if even {
            self.even.value(u, v, point)
        } else {
            self.odd.value(u, v, point)
        }
    }
}

impl CheckerTexture {
    pub fn new(scale: f64, even_color: Vec3, odd_color: Vec3) -> CheckerTexture {
        CheckerTexture {
            inv_scale: (1.0 / scale),
            even: Arc::new(SolidTexture::new(even_color)),
            odd: Arc::new(SolidTexture::new(odd_color)),
        }
    }
}
