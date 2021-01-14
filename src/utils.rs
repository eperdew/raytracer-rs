use rust_3d::{Norm3D, IsNormalized3D, Is3D};
use image::Rgb;
use std::f64::consts::TAU;

////////////////////////////////////////////////////////////////////////////////
// Rays
////////////////////////////////////////////////////////////////////////////////

pub fn reflect(ray: &Norm3D, normal: &Norm3D) -> Norm3D {
    Norm3D::new(ray * 1.0 - (normal * 2.0 * (normal.dot(ray)))).unwrap()
}

////////////////////////////////////////////////////////////////////////////////
// Colors
////////////////////////////////////////////////////////////////////////////////

pub fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> Rgb<u8> {
    // Convert the hue to degrees and clamp to [0, 360]
    let angle = (hue * 360.0 / TAU).rem_euclid(360.0);

    let c = saturation * value;
    let x = c * (1.0 - ((angle / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = value - c;

    let (rp, gp, bp) = if 0.0 <= angle && angle < 60.0 {
        (c, x, 0.0)
    } else if 60.0 <= angle && angle < 120.0 {
        (x, c, 0.0)
    } else if 120.0 <= angle && angle < 180.0 {
        (0.0, c, x)
    } else if 180.0 <= angle && angle < 240.0 {
        (0.0, x, c)
    } else if 240.0 <= angle && angle < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let (r,g,b) = ((rp + m) * 255.0, (gp + m) * 255.0, (bp + m) * 255.0);
    Rgb([ r as u8, g as u8, b as u8 ])
}