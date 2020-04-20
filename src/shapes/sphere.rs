use crate::{
    math::{Local, Point3, Ray, Vec3},
    shapes::{Intersection, Shape},
    sampling::{self, Sampler},
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

    fn local_to_world(&self, vec: Vec3<Local>) -> Vec3 {
        self.position.to_vec() + vec.coerce_system()
    }
}

impl Shape for Sphere {
    // TODO: Clean up & optimize
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

    // TODO: Clean up
    fn sample(&self, point: Point3, sampler: &mut Sampler) -> (Point3, f32) {
        let (u0, u1) = (sampler.gen_0_1(), sampler.gen_0_1());

        // Check we're not inside the sphere
        debug_assert!(self.position.distance_squared(point) > self.radius.powi(2));

        let sin_theta_max_2 = self.radius.powi(2) / self.position.distance_squared(point);
        let cos_theta_max = (1.0 - sin_theta_max_2).max(0.0).sqrt();
        let cos_theta = (1.0 - u0) + u0 * cos_theta_max;
        let sin_theta = (1.0 - cos_theta.powi(2)).max(0.0).sqrt();
        let phi = u1 * 2.0 * std::f32::consts::PI;

        let dc = self.position.distance(point);
        let ds = dc * cos_theta - (self.radius.powi(2) - (dc * sin_theta).powi(2)).max(0.0).sqrt();
        let cos_alpha = (dc.powi(2) + self.radius.powi(2) - ds.powi(2)) / (2.0 * dc * self.radius);
        let sin_alpha = (1.0 - cos_alpha.powi(2)).max(0.0).sqrt();

        let wc = (self.position - point).normalize().coerce_system();
        let (wc_x, wc_y) = wc.coordinate_system_from_unit();

        let normal = Vec3::<Local>::spherical_direction(sin_alpha, cos_alpha, phi, -wc_x, -wc_y, -wc);

        let sampled_point_local = self.radius * 1.001 * normal.normalize();
        let sampled_point_world = self.local_to_world(sampled_point_local).to_point();

        debug_assert!(sampled_point_local.len() <= self.radius * 1.01);

        (sampled_point_world, sampling::pdf_cone(cos_theta_max)) 
    }
}
