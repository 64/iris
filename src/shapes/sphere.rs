use crate::{
    math::{Point3, Ray, Vec3},
    shapes::{Intersection, Shape},
};

#[derive(Debug, Clone)]
pub struct Sphere {
    position: Point3,
    radius: f32,
}

impl Sphere {
    pub fn new(position: Point3, radius: f32) -> Self {
        Self { position, radius }
    }
}

impl Shape for Sphere {
    // TODO: Clean up
    fn intersect(&self, ray: &Ray) -> Option<(Intersection, f32)> {
        let oc = ray.o() - self.position;
        let a = ray.d().len_squared();
        let half_b = ray.d().dot(oc);
        let c = oc.len_squared() - self.radius.powi(2);
        let discrim = half_b.powi(2) - a * c;

        if discrim > 0.0 {
            let root = discrim.sqrt();
            let temp = (-half_b - root) / a;
            if temp > 0.0 {
                let point = ray.point_at(temp);
                let normal = (point - self.position) / self.radius;
                let tangeant = Vec3::new(0.0, 1.0, 0.0).cross(normal).normalize();
                let bitangeant = normal.cross(tangeant);
                return Some((
                    Intersection {
                        point,
                        normal,
                        tangeant,
                        bitangeant,
                    },
                    temp,
                ));
            }

            let temp = (-half_b + root) / a;
            if temp > 0.0 {
                let point = ray.point_at(temp);
                let normal = (point - self.position) / self.radius;
                let tangeant = Vec3::new(0.0, 1.0, 0.0).cross(normal).normalize();
                let bitangeant = normal.cross(tangeant);
                return Some((
                    Intersection {
                        point,
                        normal,
                        tangeant,
                        bitangeant,
                    },
                    temp,
                ));
            }
        }

        None
    }
}
