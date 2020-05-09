#![allow(dead_code)]
#![allow(unused)]
use crate::{
    bsdf::SampleableBsdf,
    math::{self, PdfSet, Shading, Vec3, Vec4},
    sampling::{self, Sampler},
    spectrum::{SampleableSpectrum, SpectralSample, Spectrum, Wavelength},
};

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct FresnelBsdf {
    reflected_color: Spectrum,
    transmitted_color: Spectrum,
    base_ior: f32,
    dispersion: f32,
}

impl FresnelBsdf {
    pub fn new<S: Into<Spectrum>, T: Into<Spectrum>>(s: S, t: T, base_ior: f32, dispersion: f32) -> Self {
        Self {
            reflected_color: s.into(),
            transmitted_color: t.into(),
            base_ior,
            dispersion,
        }
    }

    fn refractive_index(&self, wavelength: Wavelength) -> Vec4 {
        //1.5220 + 0.00459 / (wavelength.inner * wavelength.inner * 1e-6)
        self.base_ior + self.dispersion / (wavelength.inner * wavelength.inner * 1e-6)
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
        // TODO: SIMD this
        let eta_a = 1.0;
        let eta_b = self.refractive_index(wavelength);
        let fresnel = Vec4::new(
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.x()),
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.y()),
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.z()),
            math::fresnel_dielectric(wo.cos_theta(), eta_a, eta_b.w()),
        );

        if sampler.gen_0_1() < fresnel.hero() {
            let wi = Vec3::new(-wo.x(), -wo.y(), wo.z());
            let bsdf = self.reflected_color.evaluate(wavelength) / wi.cos_theta().abs();
            (
                wi,
                SpectralSample::from(bsdf.inner * fresnel),
                PdfSet::from(fresnel),
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
                    self.transmitted_color.evaluate(wavelength) * (1.0 - fresnel.hero()) * eta_i.powi(2)
                        / eta_t.powi(2);
                let hero_value = ft.hero() / wi.cos_theta().abs();
                (
                    wi,
                    SpectralSample::new(hero_value, 0.0, 0.0, 0.0),
                    PdfSet::new(1.0 - fresnel.hero(), 0.0, 0.0, 0.0),
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
