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
pub struct SpecularBsdf {
    reflected_color: Spectrum,
    eta: f32,
}

impl SpecularBsdf {
    pub fn new<S: Into<Spectrum>>(s: S, eta: f32) -> Self {
        Self {
            reflected_color: s.into(),
            eta,
        }
    }
}

impl SampleableBsdf for SpecularBsdf {
    fn evaluate(
        &self,
        wi: Vec3<Shading>,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
    ) -> SpectralSample {
        SpectralSample::splat(0.0)
    }

    fn pdf(&self, wi: Vec3<Shading>, wo: Vec3<Shading>, hero_wavelength: Wavelength) -> PdfSet {
        PdfSet::splat(0.0)
    }

    fn sample(
        &self,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> (Vec3<Shading>, SpectralSample, PdfSet) {
        let wi = Vec3::new(-wo.x(), -wo.y(), wo.z());
        let bsdf = self.reflected_color.evaluate(hero_wavelength)
            //* math::fresnel_dielectric(wi.cos_theta().abs(), self.eta, 1.0)
            / wi.cos_theta().abs();
        (wi, bsdf, PdfSet::splat(1.0))
    }

    fn is_specular(&self) -> bool {
        true
    }
}
