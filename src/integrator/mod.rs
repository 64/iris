use crate::{
    math::Ray,
    sampling::Sampler,
    spectrum::{SpectralSample, Wavelength},
    scene::Scene,
};

pub mod swss_slow;
pub mod hwss_slow;
pub mod swss_naive;
pub mod hwss_naive;

pub trait Integrator {
    fn radiance(&self, scene: &Scene, ray: Ray, wavelength: Wavelength, sampler: &mut Sampler) -> SpectralSample;
}
