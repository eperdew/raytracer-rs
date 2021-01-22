use rust_3d::*;
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

#[derive(Clone, Debug)]
pub struct PointLight {
    pub origin: Point3D,
    pub intensity: f64,
}

impl PointLight {
    pub fn get_intensity(&self, ray: &Line3D) -> f64 {
        let dir_from_ray = Norm3D::new(&self.origin - &ray.anchor);
        if dir_from_ray.is_err() {
            return 0.0;
        }

        let cos = dir_from_ray.unwrap().dot(&ray.dir);
        let sqr_distance = (&self.origin as &dyn Is3D).sqr_distance(&ray.anchor)
                .get();

        if sqr_distance == 0.0 {
            1.0
        } else {
            self.intensity * cos / sqr_distance
        }
    }
}