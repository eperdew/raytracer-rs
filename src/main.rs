
pub mod objects;
pub mod light;
pub mod utils;
pub mod scene;

extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;

use objects::*;
use light::*;
use utils::*;
use scene::*;

use rust_3d::*;
use std::f64::consts::TAU;

#[macro_use] extern crate itertools;
use image::{RgbaImage, Rgba};

fn main() {
    render_demo();
}

fn make_scene(t: f64) -> Scene {
    let num_spheres = 6;
    let circle_radius = 2.0;

    let mut spheres: Vec<Box<dyn Intersectable>> = Vec::new();
    for n in 0..num_spheres {
        let angle = (n as f64 / num_spheres as f64) * TAU;

        let x = circle_radius * (angle + t).cos();
        let y = circle_radius * (angle + t).sin();

        let color = hsv_to_rgb(angle, 1.0, 1.0);
        let center = Point3D::new(x, y, 10.0 + 3.0 *
            (t * (n as f64 / num_spheres as f64) as f64).sin());

        let sphere = SphereObject {
            center: center,
            radius: Positive::new(1.0).unwrap(),

            color: color,
        };
        spheres.push(Box::new(sphere));
    }

    let big_boy = SphereObject {
        center: Point3D::new(0.0, 0.0, 20.0),
        radius: Positive::new(8.0).unwrap(),

        color: hsv_to_rgb(0.5 , 1.0, 1.0),
    };
    spheres.push(Box::new(big_boy));

    let lights = vec![
        DirectionalLight {
            dir: Norm3D::new(Point3D::new(0.0, 0.0, 1.0)).unwrap(),
            color: Rgba([ 255, 255, 255, 255 ]),
        },
        DirectionalLight {
            dir: Norm3D::new(Point3D::new(1.0, -1.0, -1.0)).unwrap(),
            color: Rgba([ 255, 255, 255, 255 ]),
        },
    ];

    Scene {
        objects: spheres,
        lights,
    }
}

fn render_demo() {
    // SDL stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let (width, height) = (200, 200);
    let (window_width, window_height) = (width * 2, height * 2);

    let window = video_subsystem
        .window("Raytracer", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mouse = sdl_context.mouse();
    mouse.set_relative_mouse_mode(true);

    let mut image = RgbaImage::new(width, height);
    let mut t = 0.0;
    let mut camera = Camera {
        width,
        height,

        rot_x: Rad { val: 0.0 },
        rot_y: Rad { val: 0.0 },
        location: Point3D::new(0.0, 0.0, 0.0),
    };

    let mut paused = false;

    let mut w_pressed = false;
    let mut a_pressed = false;
    let mut s_pressed = false;
    let mut d_pressed = false;

    'running: loop {
        // Construct the scene for this frame.
        let scene = make_scene(t);

        // Render the scene.
        scene.render(&mut image, &camera);

        // Update the canvas with the image.
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
                PixelFormatEnum::ABGR8888,
                TextureAccess::Streaming, width, height).unwrap();
        texture.update(None, image.as_flat_samples().samples,
            width as usize * 4).expect("Failed to update texture");

        canvas.copy(&texture, None, None).expect("Failed to copy texture");

        // TODO: This should really be computed based on the actual time elapsed.
        let elapsed = 0.01;
        let camera_speed = 10.0;
        let turning_speed = 0.1;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                // TODO: Macro? HashMap?
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    w_pressed = true;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    a_pressed = true;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    s_pressed = true;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    d_pressed = true;
                },

                Event::KeyUp { keycode: Some(Keycode::W), .. } => {
                    w_pressed = false;
                },
                Event::KeyUp { keycode: Some(Keycode::A), .. } => {
                    a_pressed = false;
                },
                Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                    s_pressed = false;
                },
                Event::KeyUp { keycode: Some(Keycode::D), .. } => {
                    d_pressed = false;
                },

                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    paused = !paused;
                },

                Event::MouseMotion { xrel: dx, yrel: dy, .. } => {
                    camera.rotate_y( dx as f64 * elapsed * turning_speed);
                    camera.rotate_x(-dy as f64 * elapsed * turning_speed);
                },
                _ => {}
            }
        }

        if w_pressed {
            camera.move_forward(elapsed * camera_speed);
        }
        if a_pressed {
            camera.move_right(-elapsed * camera_speed);
        }
        if s_pressed {
            camera.move_forward(-elapsed * camera_speed);
        }
        if d_pressed {
            camera.move_right(elapsed * camera_speed);
        }

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        if !paused {
            t += elapsed;
        }
    }
}
