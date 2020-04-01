use super::{HitInfo, Shape};
use crate::math::{Point3, Ray, Vec3};
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
};

pub struct Sphere {
    position: Point3,
    radius: f32,
    node_index: usize,
}

impl Sphere {
    pub fn new(position: Point3, radius: f32) -> Self {
        Self {
            position,
            radius,
            node_index: 0,
        }
    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<(HitInfo, f32)> {
        let oc = ray.o - self.position;
        let a = ray.d.len_squared();
        let half_b = ray.d.dot(oc);
        let c = oc.len_squared() - self.radius.powi(2);
        let discrim = half_b.powi(2) - a * c;

        if discrim > 0.0 {
            let root = discrim.sqrt();
            let temp = (-half_b - root) / a;
            if temp > 0.0 {
                let point = ray.point_at(temp);
                return Some((
                    HitInfo {
                        point,
                        normal: (self.position - point).normalized(),
                    },
                    temp,
                ));
            }

            let temp = (-half_b + root) / a;
            if temp > 0.0 {
                let point = ray.point_at(temp);
                return Some((
                    HitInfo {
                        point,
                        normal: (self.position - point).normalized(),
                    },
                    temp,
                ));
            }
        }

        None
    }
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vec3::new(self.radius, self.radius, self.radius);
        let min = self.position - half_size;
        let max = self.position + half_size;
        AABB::with_bounds(min.to_nalgebra(), max.to_nalgebra())
    }
}

impl BHShape for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
