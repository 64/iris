use enum_dispatch::enum_dispatch;

use crate::math::{Point3, Ray, Shading, Vec3, World};

mod sphere;
pub use sphere::Sphere;

pub struct Intersection {
    pub point: Point3,
    pub normal: Vec3,
    pub tangeant: Vec3,
    pub bitangeant: Vec3,
}

impl Intersection {
    pub fn world_to_shading(&self, w: Vec3<World>) -> Vec3<Shading> {
        Vec3::new(
            self.bitangeant.dot(w),
            self.tangeant.dot(w),
            self.normal.dot(w),
        )
    }

    pub fn shading_to_world(&self, s: Vec3<Shading>) -> Vec3<World> {
        let x = self.bitangeant.x() * s.x() + self.tangeant.x() * s.y() + self.normal.x() * s.z();
        let y = self.bitangeant.y() * s.x() + self.tangeant.y() * s.y() + self.normal.y() * s.z();
        let z = self.bitangeant.z() * s.x() + self.tangeant.z() * s.y() + self.normal.z() * s.z();
        Vec3::new(x, y, z)
    }
}

#[enum_dispatch]
pub trait Shape {
    fn intersect(&self, ray: &Ray) -> Option<(Intersection, f32)>;
}

#[enum_dispatch(Shape)]
#[derive(Debug, Clone)]
pub enum Geometry {
    Sphere,
}
