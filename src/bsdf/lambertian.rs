#![allow(dead_code)]
#![allow(unused)]
use crate::{
    bsdf::SampleableBsdf,
    math::{Shading, Vec3},
    sampling::{self, Sampler},
    spectrum::{SampleableSpectrum, SpectralSample, Spectrum, Wavelength},
};

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct LambertianBsdf {
    albedo: Spectrum,
}

impl LambertianBsdf {
    pub fn new<S: Into<Spectrum>>(s: S) -> Self {
        Self { albedo: s.into() }
    }
}

impl SampleableBsdf for LambertianBsdf {
    fn evaluate(
        &self,
        wi: Vec3<Shading>,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
    ) -> SpectralSample {
        self.albedo.evaluate(hero_wavelength) / PI
    }

    fn pdf(&self, wi: Vec3<Shading>, wo: Vec3<Shading>, hero_wavelength: Wavelength) -> [f32; 4] {
        [sampling::pdf_cosine_unit_hemisphere(wi.cos_theta()); 4]
    }

    fn sample(
        &self,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> (Vec3<Shading>, [f32; 4]) {
        let wi = sampling::cosine_unit_hemisphere(sampler.gen_0_1(), sampler.gen_0_1());
        (wi, self.pdf(wi, wo, hero_wavelength))
    }
}
