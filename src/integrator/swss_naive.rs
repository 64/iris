#[allow(unused)]
use crate::{
    bsdf::{Bsdf, FresnelBsdf, LambertianBsdf, MicrofacetBsdf, SampleableBsdf, SpecularBsdf},
    integrator::Integrator,
    math::Ray,
    math::{PdfSet, Point3, Shading, Vec3},
    sampling::Sampler,
    sampling::{self, mis},
    scene::Scene,
    shape::{Geometry, Intersection, Primitive, Shape, Sphere},
    spectrum::{upsample::UpsampleTable, ConstantSpectrum, Spectrum, UpsampledHdrSpectrum},
    spectrum::{SampleableSpectrum, SpectralSample, Wavelength},
    types::PrimIndex,
};

const MAX_DEPTH: u32 = 15;
const MIN_DEPTH: u32 = 2;

pub struct SwssNaive;

impl Default for SwssNaive {
    #[allow(dead_code)]
    fn default() -> Self {
        Self
    }
}

impl Integrator for SwssNaive {
    fn radiance(
        &self,
        scene: &Scene,
        mut ray: Ray,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut radiance = SpectralSample::splat(0.0);
        let mut throughput = SpectralSample::splat(1.0);

        for bounces in 0..MAX_DEPTH {
            let (prim, hit) = match scene.intersection(&ray) {
                Some(ph) => ph,
                None => break,
            };

            let bsdf = match prim.get_material(&scene.materials) {
                Some(bsdf) => bsdf,
                None => break, // TODO: Should this break be below?
            };

            if bounces == 0 {
                // We didn't do NEE last step, accumulate light directly
                if let Some(light) = prim.get_light(&scene.lights) {
                    radiance += throughput * light.evaluate(wavelength);
                }
            }

            // Calculate direct lighting (next event estimation)
            radiance +=
                throughput * self.direct_light(bsdf, &hit, scene, &ray, wavelength, sampler);

            // Calculate indirect lighting - generate next ray direction
            let shading_wo = hit.world_to_shading(-ray.d());
            let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) =
                bsdf.sample(shading_wo, wavelength, sampler);
            let cos_theta = bsdf_sampled_wi.cos_theta().abs();
            if bsdf_pdfs.hero() == 0.0 || cos_theta == 0.0 {
                break;
            }

            throughput *= bsdf_values * cos_theta / bsdf_pdfs.hero();

            // Russian roulette
            if bounces >= MIN_DEPTH {
                let p = throughput.sum().min(0.95);
                if sampler.gen_0_1() > p {
                    break;
                }

                throughput /= SpectralSample::splat(p);
            }

            // Spawn new ray
            let world_wi = hit.shading_to_world(bsdf_sampled_wi);
            ray = Ray::spawn(hit.point, world_wi, hit.normal);
        }

        SpectralSample::new(radiance.hero(), 0.0, 0.0, 0.0)
    }
}

impl SwssNaive {
    fn direct_light(
        &self,
        bsdf: &Bsdf,
        hit: &Intersection,
        scene: &Scene,
        ray: &Ray,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut radiance = SpectralSample::splat(0.0);

        let shading_wo = hit.world_to_shading(-ray.d());
        let (light_spectrum, light_prim, light_pick_weight) = scene.pick_one_light(sampler);
        let light_emission = light_spectrum.evaluate(wavelength);

        // Sample light
        {
            let (light_pos, light_pdf) = light_prim.sample(&hit, sampler);

            let ray_to_light = Ray::spawn_to(hit.point, light_pos, hit.normal);
            let facing_forward = (light_pos - hit.point).dot(hit.normal) > 0.0;

            // Check that the light has a non-zero contribution
            // These checks are very important otherwise lights will illuminate themselves
            if light_pdf > 0.0
                && facing_forward != hit.back_face
                && light_pos.distance_squared(hit.point) > 0.00001
                && scene.ray_hits_point(&ray_to_light, light_pos)
            {
                // Add light sample contribution
                let shading_wi = hit.world_to_shading(ray_to_light.d());
                let bsdf_values = bsdf.evaluate(shading_wi, shading_wo, wavelength);
                let bsdf_pdfs = bsdf.pdf(shading_wi, shading_wo, wavelength);
                let cos_theta = shading_wi.cos_theta().abs();

                // Balance heuristic
                let mis_weight = light_pdf / (bsdf_pdfs.hero() + light_pdf);
                radiance += mis_weight * light_emission * bsdf_values * cos_theta / light_pdf;
            }
        }

        // Sample BSDF
        {
            let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) =
                bsdf.sample(shading_wo, wavelength, sampler);
            let cos_theta = bsdf_sampled_wi.cos_theta().abs();
            let ray_to_light =
                Ray::spawn(hit.point, hit.shading_to_world(bsdf_sampled_wi), hit.normal);

            // Check that the light has a non-zero contribution
            if bsdf_pdfs.hero() > 0.0 && scene.ray_hits_object(&ray_to_light, light_prim) {
                // Add light sample contribution
                let light_pdf = light_prim.pdf(&hit, ray_to_light.d());
                let mis_weight = bsdf_pdfs.hero() / (bsdf_pdfs.hero() + light_pdf);
                radiance +=
                    mis_weight * light_emission * bsdf_values * cos_theta / bsdf_pdfs.hero();
            }
        }

        radiance * light_pick_weight
    }
}
