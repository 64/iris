#![allow(dead_code)]
#![allow(unused)]
use crate::{
    bsdf::SampleableBsdf,
    math::{PdfSet, Shading, Vec3},
    sampling::{self, Sampler},
    spectrum::{SampleableSpectrum, SpectralSample, Spectrum, Wavelength},
};

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct SpecularBsdf {
    reflected_color: Spectrum,
}

impl SpecularBsdf {
    pub fn new<S: Into<Spectrum>>(s: S) -> Self {
        Self {
            reflected_color: s.into(),
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
        (Vec3::splat(0.0), SpectralSample::splat(0.0), PdfSet::splat(0.0)) 
    }

    fn is_specular(&self) -> bool {
        true
    }
}
