
pub mod objects;
pub mod light;

extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use rayon::prelude::*;

use objects::*;
use light::*;

use float_ord::FloatOrd;
use rand::prelude::*;

use rust_3d::*;
use std::f64::consts::TAU;

#[macro_use] extern crate itertools;
use image::{RgbaImage, Rgb, Rgba, Pixel};

fn main() {
    render_demo();
}

fn pixel_to_unit_square(x: u32, y: u32, width: f64, height: f64) -> Line3D {
    let x_prime = x as f64 / width - 0.5;
    let y_prime = y as f64 / height - 0.5;

    Line3D::new(
        Point3D::default(),
        Norm3D::new(Point3D::new(x_prime, y_prime, 1.0)).unwrap())
}

fn get_ambient_light(ray: &Line3D, lights: &[DirectionalLight]) -> Rgba<u8> {
    // Compute the color of the light based on the light sources in the scene.
    let mut result = Rgba([ 0, 0, 0, 0 ]);
    for light in lights {
        light.colorize(&ray.dir, &mut result);
    }

    result
}

fn reflect(ray: &Norm3D, normal: &Norm3D) -> Norm3D {
    Norm3D::new(ray * 1.0 - (normal * 2.0 * (normal.dot(ray)))).unwrap()
}

fn raytrace(ray: &Line3D, objects: &[SphereObject],
        lights: &[DirectionalLight], max_depth: isize) -> Rgba<u8>
{
    if max_depth <= 0 {
        // If we've run out of gas, we haven't hit a light source and this pixel
        // is just black.
        return Rgba([0, 0, 0, 0]);
    }

    // Find the closest intersection
    let intersection = objects.iter()
        // Filter to those objects that intersect
        .filter_map(|o| o.intersect(ray)
            // And remember the object, intersection, and distance measure
            .map(|v| (o, v.clone(), FloatOrd(sqr_dist_3d(&ray.anchor, &v)))))
        // Grab the closest intersection (smallest measure)
        .min_by(|(_, _, dist1), (_, _, dist2)| dist1.cmp(dist2))
        // And throw away the measure.
        .map(|(o, v, _)| (o, v));

    if intersection.is_none() {
        // No intersection, so color this based on the ambient light.
        return get_ambient_light(ray, lights);
    }

    let (object, intersection) = intersection.unwrap();
    let normal = object.normal(&intersection);

    if let Some(normal) = normal {
        // We actually have a normal - do a bounce.
        let reflected_dir = reflect(&ray.dir, &normal);

        // Do a very slightly weird thing here - march the ray slightly forward.
        let new_anchor = intersection + &reflected_dir * 0.0001;
        let subray = Line3D::new(new_anchor, reflected_dir);

        let subcolor = raytrace(&subray, objects, lights, max_depth - 1);

        // Blend this color onto our object.
        let (ro, go, bo, _) = object.get_color().channels4();
        let (r, g, b, a) = subcolor.channels4();

        let r_prop = r as f64 * a as f64 / (255.0 * 255.0);
        let g_prop = g as f64 * a as f64 / (255.0 * 255.0);
        let b_prop = b as f64 * a as f64 / (255.0 * 255.0);

        let r_result = ro as f64 * r_prop;
        let b_result = bo as f64 * b_prop;
        let g_result = go as f64 * g_prop;

        let result = Rgba([ r_result as u8, g_result as u8, b_result as u8, a ]);

        result
    } else {
        // Something went wrong computing the normal, so just give ambient light.
        get_ambient_light(ray, lights)
    }
}

fn render_demo() {
    let circle_radius = 2.0;

    let lights: &[DirectionalLight] = {
        let light1 = DirectionalLight {
            dir: Norm3D::new(Point3D::new(0.0, 0.0, 1.0)).unwrap(),
            color: Rgba([ 255, 255, 255, 255 ]),
        };

        let light2 = DirectionalLight {
            dir: Norm3D::new(Point3D::new(1.0, -1.0, -1.0)).unwrap(),
            color: Rgba([ 255, 255, 255, 255 ]),
        };

        &[light1, light2]
    };

    // SDL stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let (width, height) = (600, 600);

    let window = video_subsystem.window("Raytracer", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut image = RgbaImage::new(width, height);
    let mut t = 0.0;

    let num_spheres = 6;

    let mut spheres = Vec::new();
    for n in 0..num_spheres {
        let x = circle_radius * (n as f64 / (num_spheres as f64) * TAU).cos();
        let y = circle_radius * (n as f64 / (num_spheres as f64) * TAU).sin();

        let sphere = SphereObject {
            center: Point3D::new(x, y, 10.0),
            radius: Positive::new(1.0).unwrap(),

            color: Rgb([ random::<u8>(), random::<u8>(), random::<u8>() ]),
        };
        spheres.push(sphere);
    }

    let big_boy = SphereObject {
        center: Point3D::new(0.0, 0.0, 20.0),
        radius: Positive::new(8.0).unwrap(),

        color: Rgb([ random::<u8>(), random::<u8>(), random::<u8>() ]),
    };
    spheres.push(big_boy);

    'running: loop {

        for n in 0..num_spheres {
            let sphere: &mut SphereObject = &mut spheres.get_mut(n).unwrap();

            let x = circle_radius * (n as f64 / (num_spheres as f64) * TAU + t).cos();
            let y = circle_radius * (n as f64 / (num_spheres as f64) * TAU + t).sin();

            sphere.center = Point3D::new(x, y, 10.0 + 3.0 *
                    (t * (n as f64 / num_spheres as f64) as f64).sin());
        }

        // let mut objects: Vec<&dyn Intersectable> = Vec::new();
        // for sphere in &spheres {
        //     objects.push(sphere);
        // }

        // Render the scene.
        render(&spheres, &lights, &mut image, width, height);

        // Update the canvas with the image.
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
                PixelFormatEnum::ABGR8888,
                TextureAccess::Streaming, width, height).unwrap();
        texture.update(None, image.as_flat_samples().samples,
            width as usize * 4).expect("Failed to update texture");

        canvas.copy(&texture, None, None).expect("Failed to copy texture");

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        t += 0.1;
    }
}


fn render(objects: &[SphereObject], lights: &[DirectionalLight],
        image: &mut RgbaImage, width: u32, height: u32)
{
    let coords: Vec<(u32,u32)> = iproduct!(0..width, 0..height).collect();

    let mut colors = Vec::new();
    coords.par_iter()
        .map(|(x, y)| (x, y, pixel_to_unit_square(*x, *y, width as f64, height as f64)))
        .map(|(x, y, ray)| (x, y, raytrace(&ray, &objects, &lights, 10)))
        .collect_into_vec(&mut colors);

    for (x, y, color) in colors {
        let mut result = Rgba([ 0, 0, 0, 255 ]);
        result.blend(&color);

        image.put_pixel(*x, *y, result);
    }
}
