use crate::{
    math::{mis, Point3, Ray},
    sampler::Sampler,
    shapes::{Shape, Sphere},
    spectrum::{SpectrumSample, Wavelength},
    upsample::{UpsampleTable, UpsampledSpectrum},
};
use bvh::bvh::BVH;

pub struct Scene {
    bvh: BVH,
    spheres: Vec<Sphere>,
    sphere_color: UpsampledSpectrum,
}

impl Scene {
    pub fn dummy() -> Self {
        let mut spheres = vec![Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.7)];
        let bvh = BVH::build(&mut spheres);
        let upsample_table = UpsampleTable::load();

        Self {
            spheres,
            bvh,
            sphere_color: upsample_table.get_spectrum([0.7, 0.0, 0.0]),
        }
    }

    pub fn trace_ray(
        &self,
        ray: Ray,
        hero_wavelength: Wavelength,
        _sampler: &mut Sampler,
    ) -> SpectrumSample {
        let pdfs = [
            hero_wavelength.rotate_n(0).pdf(),
            hero_wavelength.rotate_n(1).pdf(),
            hero_wavelength.rotate_n(2).pdf(),
            hero_wavelength.rotate_n(3).pdf(),
        ];

        // Hero wavelength spectral sampling
        // TODO: Combine with other PT MIS techniques
        let mis_weight = mis::hwss_weight(pdfs[0], 1.0, pdfs, [1.0, 1.0, 1.0, 1.0]);

        let hit = self
            .bvh
            .traverse(
                &ray.to_nalgebra(),
                &self.spheres,
            )
            .iter()
            .filter_map(|sphere| sphere.intersect(&ray))
            .min_by_key(|(_hit, ray_t)| ordered_float::NotNan::new(*ray_t).unwrap())
            .map(|(hit, _ray_t)| hit);

        let sample = match hit {
            Some(_hit) => SpectrumSample::from_spectrum(hero_wavelength, |lambda| {
                self.sphere_color.evaluate(lambda) / 100.0
            }),
            None => SpectrumSample::splat((ray.o.y + 1.0) / 200.0),
        };

        sample * mis_weight
    }
}
