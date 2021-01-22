
use crate::light::*;
use crate::objects::*;
use crate::utils::*;

use std::f64::consts::TAU;
use image::{Rgba, Pixel, RgbaImage};
use rust_3d::*;
use rayon::prelude::*;
use float_ord::FloatOrd;

////////////////////////////////////////////////////////////////////////////////
// Camera
////////////////////////////////////////////////////////////////////////////////

pub struct Camera {
    pub width: u32,
    pub height: u32,

    pub rot_theta: Rad,
    pub rot_psi: Rad,
    pub location: Point3D,
}

impl Camera {
    fn rotation_matrix(&self) -> Matrix4 {
        let theta_rotation = Matrix4::rotation(Rad { val: 0.0 },
            self.rot_theta, Rad { val: 0.0 });
        let axis = Point3D::new(1.0, 0.0, 0.0)
            .transformed(&theta_rotation);
        let psi_rotation = Matrix4::rotation_axis(
            &Norm3D::new(axis).unwrap(), self.rot_psi);

        psi_rotation * theta_rotation
    }

    pub fn get_rays(&self) -> Vec<(u32, u32, Line3D)> {
        let rotation_matrix = self.rotation_matrix();

        let construct_ray = |x, y| {
            let x_prime = x as f64 / (self.width  as f64) - 0.5;
            let y_prime = -1.0 * (y as f64 / (self.height as f64) - 0.5);

            let direction = Point3D::new(x_prime, y_prime, 1.0)
                .transformed(&rotation_matrix);

            Line3D::new(
                self.location.clone(),
                Norm3D::new(direction).unwrap())
        };

        iproduct!(0..self.width, 0..self.height)
            .map(|(x, y)| (x, y, construct_ray(x, y)))
            .collect()
    }

    fn direction(&self) -> Norm3D {
        Norm3D::new(Point3D::new(0.0, 0.0, 1.0)
            .transformed(&self.rotation_matrix()))
            .unwrap()
    }

    pub fn move_forward(&mut self, amount: f64) {
        self.location = &self.location + self.direction() * amount;
    }

    pub fn move_right(&mut self, amount: f64) {
        let right = Point3D::new(1.0, 0.0, 0.0)
            .transformed(&self.rotation_matrix());
        self.location = &self.location + right * amount;
    }

    pub fn rotate_theta(&mut self, amount: f64) {
        self.rot_theta.val += amount;
    }

    pub fn rotate_psi(&mut self, amount: f64) {
        let max_vertical_rotation =  65.0 / 360.0 * TAU;

        self.rot_psi.val += amount;

        if self.rot_psi.val.abs() >= max_vertical_rotation {
            self.rot_psi = Rad { val: self.rot_psi.val.signum() * max_vertical_rotation };
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Scene
////////////////////////////////////////////////////////////////////////////////

pub struct Scene {
    pub objects: Vec<Box<dyn Intersectable>>,
    pub directional_lights: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
}

impl Scene {
    fn get_ambient_light(&self, ray: &Line3D) -> Rgba<u8> {
        // Compute the color of the light based on the light sources in the scene.
        let mut result = Rgba([ 0, 0, 0, 0 ]);
        for light in &self.directional_lights {
            light.colorize(&ray.dir, &mut result);
        }

        result
    }

    fn find_closest_intersection(&self, ray: &Line3D, max_distance: f64) ->
            Option<(&dyn Intersectable, Point3D)>
    {
        // Find the closest intersection
        self.objects.iter()
            // Filter to those objects that intersect
            .filter_map(|o| o.intersect(ray, max_distance)
                // And remember the object, intersection, and distance measure
                .map(|v| (o, v.clone(), FloatOrd(sqr_dist_3d(&ray.anchor, &v)))))
            // Grab the closest intersection (smallest measure)
            .min_by(|(_, _, dist1), (_, _, dist2)| dist1.cmp(dist2))
            // And throw away the measure.
            .map(|(o, v, _)| (o.as_ref(), v))
    }

    fn raytrace(&self, ray: &Line3D, max_depth: isize, max_distance: f64) -> Rgba<u8> {
        if max_depth <= 0 {
            // If we've run out of gas, we haven't hit a light source and this pixel
            // is just black.
            return Rgba([0, 0, 0, 0]);
        }

        let maybe_intersection = self.find_closest_intersection(ray, max_distance);
        if maybe_intersection.is_none() {
            // No intersection, so color this based on the ambient light.
            return self.get_ambient_light(ray);
        }

        let (object, intersection) = maybe_intersection.unwrap();
        let normal = object.normal(&intersection);

        if normal.is_none() {
            // Something went wrong computing the normal, so just give ambient light.
            return self.get_ambient_light(ray);
        }
        let normal = normal.unwrap();

        // We actually have a normal - do a bounce.
        let reflected_dir = reflect(&ray.dir, &normal);

        // Do a very slightly weird thing here - march the ray slightly forward.
        let new_anchor = intersection + &reflected_dir * 0.0001;
        let subray = Line3D::new(new_anchor, reflected_dir);

        let mut total_light_intensity = 0.0;
        for point_light in &self.point_lights {
            // Compute the distance to the point light.
            let distance = (&point_light.origin as &dyn Is3D)
                    .distance(&subray.anchor).get();
            let direction_to_light = Line3D::new(subray.anchor.clone(),
                Norm3D::new(&point_light.origin - &subray.anchor).unwrap());

            // See if there are any intersections between here and the light.
            let intersection = self.find_closest_intersection(
                    &direction_to_light, distance);

            if intersection.is_some() {
                continue;
            }

            // Nothing is between here and the light.
            total_light_intensity += point_light.get_intensity(
                    &direction_to_light);
        };

        if total_light_intensity > 1.0 {
            total_light_intensity = 1.0;
        }

        let (r, g, b, _) = object.get_color().channels4();

        Rgba([ r, g, b, (total_light_intensity * 255.0) as u8 ])

        ////////////////////////////////////////////////////////////////////////
        // Indirect lighting - Disabled for now
        ////////////////////////////////////////////////////////////////////////

        // let subcolor = self.raytrace(&subray, max_depth - 1, max_distance);

        // // Blend this color onto our object.
        // let (ro, go, bo, _) = object.get_color().channels4();
        // let (r, g, b, a) = subcolor.channels4();

        // let r_prop = r as f64 * a as f64 / (255.0 * 255.0);
        // let g_prop = g as f64 * a as f64 / (255.0 * 255.0);
        // let b_prop = b as f64 * a as f64 / (255.0 * 255.0);

        // let r_result = ro as f64 * r_prop;
        // let b_result = bo as f64 * b_prop;
        // let g_result = go as f64 * g_prop;

        // Rgba([ r_result as u8, g_result as u8, b_result as u8, a ])
    }

    pub fn render(&self, image: &mut RgbaImage, camera: &Camera) {
        let rays = camera.get_rays();

        let mut colors = Vec::new();
        rays.par_iter()
            .map(|(x, y, ray)| (x, y, self.raytrace(&ray, 10, 10_000.0)))
            .collect_into_vec(&mut colors);

        for (x, y, color) in colors {
            let mut result = Rgba([ 0, 0, 0, 255 ]);
            result.blend(&color);

            image.put_pixel(*x, *y, result);
        }
    }
}