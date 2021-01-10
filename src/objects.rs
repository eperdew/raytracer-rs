
use ndarray::{arr1, Array1, ArrayView1, Array2, ArrayView2};
use float_ord::FloatOrd;

pub type Vec3 = Array1<f32>;
pub type VecView3<'a> = ArrayView1<'a, f32>;

pub type Mat3 = Array2<f32>;
pub type MatView3<'a> = ArrayView2<'a, f32>;

////////////////////////////////////////////////////////////////////////////////
// Basic vector functions
////////////////////////////////////////////////////////////////////////////////

pub fn norm_squared(v: VecView3) -> f32 {
    v.dot(&v)
}

pub fn norm(v: VecView3) -> f32 {
    v.dot(&v).sqrt()
}

pub fn normalize(mut v: Vec3) -> Vec3 {
    let norm = norm(v.view());
    v.mapv_inplace(|e| e / norm);
    v
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    arr1(&[ x, y, z ])
}

pub fn zero3() -> Vec3 {
    vec3(0.0, 0.0, 0.0)
}

////////////////////////////////////////////////////////////////////////////////
// Rays
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct Ray {
    // x = origin + t * dir
    origin: Vec3,
    dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Ray {
        Ray {
            origin,
            dir: normalize(dir),
        }
    }

    pub fn at_zero(dir: Vec3) -> Ray {
        Ray::new(zero3(), dir)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Intersections
////////////////////////////////////////////////////////////////////////////////

pub trait Intersectable {
    // Returns the closest intersection of `self` with `ray`, if it exists.
    fn intersect(&self, ray: &Ray) -> Option<Vec3>;
}

////////////////////////////////////////////////////////////////////////////////
// Spheres
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Sphere {
    // || x - center ||^2 = radius^2
    pub center: Vec3,
    pub radius: f32,
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Vec3> {
        // || t * dir + origin - center || ^ 2 = radius^2
        let x = &ray.origin - &self.center;

        // t^2||dir||^2 + 2t * x . dir + ||x||^2 - radius^2 = 0

        // Solve the quadratic in t
        let a = norm_squared(ray.dir.view());
        let b = 2.0 * ray.dir.dot(&x);
        let c = norm_squared(x.view()) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0f32 {
            return None
        }

        // Compute the two intersections
        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

        // We only want t positive, since negative would indicate that the
        // intersection is behind the origin of the ray.
        let tbest = [ t1, t2 ].iter()
            .map(|f| FloatOrd(*f))
            .filter(|t| t >= &FloatOrd(0.0f32))
            .min();

        tbest.map(|t| &ray.origin + &(t.0 * &ray.dir))
    }
}