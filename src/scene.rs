use crate::{
    math::{Point3, Ray},
    sampler::Sampler,
    shapes::{Shape, Sphere},
    spectrum::{SpectrumSample, Wavelength},
};
use bvh::bvh::BVH;

pub struct Scene {
    bvh: BVH,
    spheres: Vec<Sphere>,
}

impl Scene {
    pub fn dummy() -> Self {
        let mut spheres = vec![Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.3)];
        let bvh = BVH::build(&mut spheres);

        Self { spheres, bvh }
    }

    pub fn trace_ray(
        &self,
        ray: Ray,
        _wavelength: Wavelength,
        _sampler: &mut Sampler,
    ) -> SpectrumSample {
        let ray_t = self
            .bvh
            .traverse(
                &bvh::ray::Ray::new(ray.o.to_nalgebra(), ray.d.to_nalgebra()),
                &self.spheres,
            )
            .iter()
            .filter_map(|sphere| sphere.intersect(&ray))
            .min_by_key(|ray_t| ordered_float::NotNan::new(*ray_t).unwrap());

        match ray_t {
            Some(_ray_t) => SpectrumSample::splat(0.0),
            None => SpectrumSample::splat((ray.o.y + 1.0) / 200.0),
        }

        // if ray.o.x * ray.o.x + ray.o.y * ray.o.y <= 0.6 {
        // let pdf = |_, w: Wavelength| {
        // let mean = 650.0;
        // let var = 40.0;
        // use statrs::distribution::{Continuous, Normal};
        // let dist = Normal::new(mean, var).unwrap();
        //(dist.pdf(w.as_nm_f32() as f64) / (100.0 * dist.pdf(mean))) as f32
        //};

        // SpectrumSample::splat(0.0).map(wavelength, pdf)
        //} else {
        // SpectrumSample::splat((ray.o.y + 1.0) / 200.0)
        //}
    }
}
