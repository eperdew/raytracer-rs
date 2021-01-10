
pub mod objects;
use objects::*;

#[macro_use] extern crate itertools;
use image::{GrayImage, Luma};

fn main() {
    let sphere = Sphere {
        center: vec3(0.0, 0.0, 5.0),
        radius: 1.0,
    };

    // let ray = Ray::at_zero(vec3(1.0, 0.0, 0.0));

    // println!("Sphere: {:?}", sphere);
    // println!("Ray: {:?}", ray);
    // println!("Intersection: {:?}", sphere.intersect(&ray));

    render(&sphere);
}

fn pixel_to_unit_square(x: u32, y: u32, width: f32, height: f32) -> Ray {
    let x_prime = x as f32 / width - 0.5f32;
    let y_prime = y as f32 / height - 0.5f32;

    Ray::at_zero(vec3(x_prime, y_prime, 1.0f32))
}

fn render(sphere: &Sphere) {
    let width = 480;
    let height = 480;

    let mut image = GrayImage::new(width, height);

    for (x, y) in iproduct!(0..width, 0..height) {
        let ray = pixel_to_unit_square(x, y, width as f32, height as f32);
        let intersection = sphere.intersect(&ray);

        let luma = if intersection.is_none() { Luma([0]) } else { Luma([128]) };
        image.put_pixel(x, y, luma);

        // println!("ray = {:?}, intersection = {:?}", ray, intersection);
    }

    image.save("sphere.png").expect("Failed to save image!");
}
