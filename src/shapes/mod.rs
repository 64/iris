use bvh::{aabb::Bounded, bounding_hierarchy::BHShape};

use crate::math::{Point3, Ray};

mod sphere;
pub use sphere::Sphere;

pub trait Shape: Bounded + BHShape {
    fn intersect(&self, ray: &Ray) -> Option<f32>;
}
