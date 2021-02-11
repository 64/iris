use crate::{
    math::{self, Local, Point3, Ray, Vec3},
    sampling::{self, Sampler},
    shape::{Intersection, Shape},
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
                let back_face = normal.dot(ray.d()) >= 0.0;
                return Some((
                    Intersection {
                        point,
                        normal,
                        tangeant,
                        bitangeant,
                        back_face,
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
                let back_face = normal.dot(ray.d()) >= 0.0;
                return Some((
                    Intersection {
                        point,
                        normal,
                        tangeant,
                        bitangeant,
                        back_face,
                    },
                    temp,
                ));
            }
        }

        None
    }

    // TODO: Clean up
    fn sample(&self, hit: &Intersection, sampler: &mut Sampler) -> (Point3, f32) {
        let (u0, u1) = (sampler.gen_0_1(), sampler.gen_0_1());

        // Offset so that the point lies on the right side of the sphere
        let point = if hit.back_face {
            math::offset_origin(hit.point, -hit.normal)
        } else {
            math::offset_origin(hit.point, hit.normal)
        };

        // Sample uniformly if we're inside the sphere
        if self.position.distance_squared(point) <= self.radius.powi(2) {
            let light_normal = sampling::unit_sphere(u0, u1);
            let light_point = self.position + self.radius * light_normal;
            let area = 4.0 * std::f32::consts::PI * self.radius.powi(2);
            let pdf = light_point.distance_squared(point)
                / (light_normal.dot((light_point - point).normalize()) * area);
            // Terrible hack, prevents us dividing by a tiny pdf and getting a massive
            // firefly
            return (light_point, pdf.max(0.001));
        }

        // https://github.com/mmp/pbrt-v3/blob/master/src/shapes/sphere.cpp#L232

        // Outside the sphere, sample cone
        let dc = self.position.distance(hit.point);
        let wc = (self.position - hit.point).normalize().coerce_system();
        let (wc_x, wc_y) = wc.coordinate_system_from_unit();

        let sin_theta_max = self.radius / dc;
        let sin_theta_max_2 = sin_theta_max.powi(2);
        let cos_theta_max = (1.0 - sin_theta_max_2).max(0.0).sqrt();

        let (sin_theta_2, cos_theta) = if sin_theta_max_2 < 0.00068523 {
            let sin_theta_2 = sin_theta_max_2 * u0;
            (sin_theta_2, (1.0 - sin_theta_2).max(0.0).sqrt())
        } else {
            let cos_theta = (cos_theta_max - 1.0) * u0 + 1.0;
            (1.0 - cos_theta.powi(2), cos_theta)
        };

        let cos_alpha = sin_theta_2 / sin_theta_max
            + cos_theta * (1.0 - sin_theta_2 / sin_theta_max_2).max(0.0).sqrt();
        let sin_alpha = (1.0 - cos_alpha.powi(2)).max(0.0).sqrt();
        let phi = u1 * 2.0 * std::f32::consts::PI;

        let normal =
            Vec3::<Local>::spherical_direction(sin_alpha, cos_alpha, phi, -wc_x, -wc_y, -wc);

        // TODO: How come this isn't quite normalized?
        let sampled_point_local = self.radius * normal.normalize();
        let sampled_point_world = self.local_to_world(sampled_point_local).to_point();

        // dbg!(normal);
        debug_assert!(sampled_point_world.distance(self.position) >= self.radius * 0.98);
        debug_assert!(sampled_point_world.distance(self.position) <= self.radius * 1.02);

        (sampled_point_world, sampling::pdf_cone(cos_theta_max))
    }

    fn pdf(&self, hit: &Intersection, wi: Vec3) -> f32 {
        // Offset so that the point lies on the right side of the sphere
        let point = if hit.back_face {
            math::offset_origin(hit.point, -hit.normal)
        } else {
            math::offset_origin(hit.point, hit.normal)
        };

        // Intersect with geometry
        if self.position.distance_squared(point) <= self.radius.powi(2) {
            let ray = Ray::spawn(hit.point, wi, hit.normal);
            if let Some((light_hit, _)) = self.intersect(&ray) {
                let area = 4.0 * std::f32::consts::PI * self.radius.powi(2);
                let pdf = light_hit.point.distance_squared(point)
                    / (light_hit.normal.dot((light_hit.point - point).normalize()) * area);
                // return pdf;
                return pdf.max(0.001); // Pretty terrible hack
            } else {
                return 0.0;
            }
        }

        let sin_theta_max_2 = self.radius.powi(2) / self.position.distance_squared(hit.point);
        let cos_theta_max = (1.0 - sin_theta_max_2).max(0.0).sqrt();
        sampling::pdf_cone(cos_theta_max)
    }
}
