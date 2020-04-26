#![allow(dead_code)]
use crate::{
    bsdf::SampleableBsdf,
    math::{Shading, Vec3},
    sampling::Sampler,
    spectrum::{SpectralSample, Wavelength},
};

#[derive(Debug, Clone)]
pub struct NullBsdf;

impl NullBsdf {
    pub fn new() -> Self {
        Self
    }
}

impl SampleableBsdf for NullBsdf {
    fn evaluate(
        &self,
        _wi: Vec3<Shading>,
        _wo: Vec3<Shading>,
        _hero_wavelength: Wavelength,
    ) -> SpectralSample {
        unreachable!()
    }

    fn pdf(
        &self,
        _wi: Vec3<Shading>,
        _wo: Vec3<Shading>,
        _hero_wavelength: Wavelength,
    ) -> [f32; 4] {
        unreachable!()
    }

    fn sample(
        &self,
        _wo: Vec3<Shading>,
        _hero_wavelength: Wavelength,
        _sampler: &mut Sampler,
    ) -> (Vec3<Shading>, [f32; 4]) {
        unreachable!()
    }
}
