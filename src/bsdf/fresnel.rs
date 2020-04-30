#![allow(dead_code)]
#![allow(unused)]
use crate::{
    bsdf::SampleableBsdf,
    math::{self, PdfSet, Shading, Vec3},
    sampling::{self, Sampler},
    spectrum::{SampleableSpectrum, SpectralSample, Spectrum, Wavelength},
};

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct FresnelBsdf {
    reflected_color: Spectrum,
    transmitted_color: Spectrum,
    eta: f32,
}

impl FresnelBsdf {
    pub fn new<S: Into<Spectrum>, T: Into<Spectrum>>(s: S, t: T, eta: f32) -> Self {
        Self {
            reflected_color: s.into(),
            transmitted_color: t.into(),
            eta,
        }
    }

    fn refractive_index(&self, wavelength: f32) -> f32 {
        //1.5220 + (0.00459 * 1e-12) / (wavelength * 1e-9).powi(2)
        1.5220 + 0.1000 / (wavelength.powi(2) * 1e-6)
    }
}

impl SampleableBsdf for FresnelBsdf {
    fn evaluate(
        &self,
        wi: Vec3<Shading>,
        wo: Vec3<Shading>,
        wavelength: Wavelength,
    ) -> SpectralSample {
        SpectralSample::splat(0.0)
    }

    fn pdf(&self, wi: Vec3<Shading>, wo: Vec3<Shading>, wavelength: Wavelength) -> PdfSet {
        PdfSet::splat(0.0)
    }

    fn sample(
        &self,
        wo: Vec3<Shading>,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> (Vec3<Shading>, SpectralSample, PdfSet) {
        // TODO: SIMD this, this is a mess
        let eta_a = 1.0;
        let eta_b = SpectralSample::from_function(wavelength, |wl| self.refractive_index(wl));
        let fresnel = [
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.x()),
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.y()),
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.z()),
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.w()),
        ];

        if sampler.gen_0_1() < fresnel[0] {
            let wi = Vec3::new(-wo.x(), -wo.y(), wo.z());
            let bsdf = self.reflected_color.evaluate(wavelength) / wi.cos_theta().abs();
            (
                wi,
                SpectralSample::new(
                    bsdf.x() * fresnel[0],
                    bsdf.y() * fresnel[1],
                    bsdf.z() * fresnel[2],
                    bsdf.w() * fresnel[3],
                ),
                PdfSet::new(fresnel[0], fresnel[1], fresnel[2], fresnel[3]),
            )
        } else {
            let (eta_i, eta_t) = if wo.cos_theta() > 0.0 {
                (eta_a, eta_b.hero())
            } else {
                (eta_b.hero(), eta_a)
            };

            if let Some(wi) =
                math::refract(wo, Vec3::new(0.0, 0.0, 1.0).face_forward(wo), eta_i / eta_t)
            {
                let ft =
                    self.transmitted_color.evaluate(wavelength) * (1.0 - fresnel[0]) * eta_i.powi(2)
                        / eta_t.powi(2);
                let hero_value = ft.hero() / wi.cos_theta().abs();
                (
                    wi,
                    SpectralSample::new(hero_value, 0.0, 0.0, 0.0),
                    PdfSet::new(1.0 - fresnel[0], 0.0, 0.0, 0.0),
                )
            } else {
                // Total internal reflection
                (
                    Vec3::splat(0.0),
                    SpectralSample::splat(0.0),
                    PdfSet::splat(0.0),
                )
            }
        }
    }

    fn is_specular(&self) -> bool {
        true
    }
}
