use crate::{
    math::Ray,
    sampling::Sampler,
    scene::Scene,
    spectrum::{SpectralSample, Wavelength},
};

pub mod hwss_naive;
pub mod hwss_slow;
pub mod swss_naive;
pub mod swss_slow;

pub trait Integrator {
    fn radiance(
        &self,
        scene: &Scene,
        ray: Ray,
        wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> SpectralSample;
}
