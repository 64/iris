use bvh::{aabb::Bounded, bounding_hierarchy::BHShape};

use crate::math::{Point3, Ray, Vec3};

mod sphere;
pub use sphere::Sphere;

pub struct HitInfo {
    pub point: Point3,
    pub normal: Vec3,
}

pub trait Shape: Bounded + BHShape {
    fn intersect(&self, ray: &Ray) -> Option<(HitInfo, f32)>;
}
