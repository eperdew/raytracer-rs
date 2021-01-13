use rust_3d::{Norm3D, Is3D};
use image::{Rgba, Pixel};

#[derive(Clone, Debug)]
pub struct DirectionalLight {
    pub dir: Norm3D,
    pub color: Rgba<u8>,
}

impl DirectionalLight {
    pub fn get_intensity(&self, ray: &Norm3D) -> f64 {
        let dotproduct = (&self.dir * -1.0).dot(ray);

        if dotproduct < 0.0 { 0.0 } else { dotproduct }
    }

    pub fn colorize(&self, ray: &Norm3D, other_color: &mut Rgba<u8>) {
        let ray_color: Rgba<u8> = self.color.map_with_alpha(
                |rgb| rgb,
                |a| (a as f64 * self.get_intensity(ray)) as u8);

        other_color.blend(&ray_color);
    }
}