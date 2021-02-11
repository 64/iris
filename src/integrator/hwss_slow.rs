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

pub struct HwssSlow;

impl Default for HwssSlow {
    #[allow(dead_code)]
    fn default() -> Self {
        Self
    }
}

impl Integrator for HwssSlow {
    fn radiance(
        &self,
        scene: &Scene,
        mut ray: Ray,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample {
        let mut radiance = SpectralSample::splat(0.0);
        let mut throughput = SpectralSample::splat(1.0);
        let mut path_pdfs = PdfSet::splat(1.0);

        for bounces in 0..MAX_DEPTH {
            let (prim, hit) = match scene.intersection(&ray) {
                Some(ph) => ph,
                None => break,
            };

            // Accumulate emission
            if let Some(light) = prim.get_light(&scene.lights) {
                radiance += throughput * light.evaluate(wavelength) * mis::balance_heuristic_1(path_pdfs);
            }

            // Sample BSDF
            let bsdf = match prim.get_material(&scene.materials) {
                Some(bsdf) => bsdf,
                None => break,
            };

            let shading_wo = hit.world_to_shading(-ray.d());
            let (bsdf_sampled_wi, bsdf_values, bsdf_pdfs) =
                bsdf.sample(shading_wo, wavelength, sampler);
            let cos_theta = bsdf_sampled_wi.cos_theta().abs();
            if bsdf_pdfs.hero() == 0.0 || cos_theta == 0.0 {
                break;
            }

            throughput *= bsdf_values * cos_theta / bsdf_pdfs.hero();
            path_pdfs *= bsdf_pdfs;

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

        radiance
    }
}
