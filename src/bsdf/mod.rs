use crate::{
    math::{Shading, Vec3},
    sampling::Sampler,
    spectrum::{SpectralSample, Wavelength},
};
use enum_dispatch::enum_dispatch;

mod lambertian;
pub use lambertian::LambertianBsdf;

mod null;
pub use null::NullBsdf;

#[enum_dispatch]
pub trait SampleableBsdf {
    fn evaluate(
        &self,
        wi: Vec3<Shading>,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
    ) -> SpectralSample;

    // Returns the sampled direction as well as the PDF for each wavelength
    fn sample(
        &self,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> (Vec3<Shading>, [f32; 4]);
}

#[enum_dispatch(SampleableBsdf)]
#[derive(Debug, Clone)]
pub enum Bsdf {
    LambertianBsdf,
    NullBsdf,
}
