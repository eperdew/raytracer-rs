use rust_3d::*;
use float_ord::FloatOrd;
use image::Rgb;

////////////////////////////////////////////////////////////////////////////////
// Intersections
////////////////////////////////////////////////////////////////////////////////

pub trait Intersectable: Send + Sync {
    // Returns the closest intersection of `self` with `ray`, if it exists.
    fn intersect(&self, ray: &Line3D) -> Option<Point3D>;
    fn normal(&self, point: &Point3D) -> Option<Norm3D>;
    fn get_color(&self) -> Rgb<u8>;
}

////////////////////////////////////////////////////////////////////////////////
// Spheres
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct SphereObject {
    pub radius: Positive,
    pub center: Point3D,

    pub color: Rgb<u8>,
}

impl Intersectable for SphereObject {
    fn intersect(&self, ray: &Line3D) -> Option<Point3D> {
        // || t * dir + origin - center || ^ 2 = radius^2
        let x = &ray.anchor - &self.center;

        // t^2||dir||^2 + 2t * x . dir + ||x||^2 - radius^2 = 0

        // Solve the quadratic in t
        let a = ray.dir.dot(&ray.dir);
        let b = 2.0 * ray.dir.dot(&x);
        let c = x.dot(&x) - self.radius.get() * self.radius.get();

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None
        }

        // Compute the two intersections
        let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

        // We only want t positive, since negative would indicate that the
        // intersection is behind the origin of the ray.
        let tbest = [ t1, t2 ].iter()
            .map(|f| FloatOrd(*f))
            .filter(|t| t >= &FloatOrd(0.0))
            .min();

        tbest.map(|t| &ray.anchor + &(&ray.dir * t.0))
    }

    fn normal(&self, point: &Point3D) -> Option<Norm3D> {
        Norm3D::new(point - &self.center).ok()
    }

    fn get_color(&self) -> Rgb<u8> {
        self.color.clone()
    }
}