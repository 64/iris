use enum_dispatch::enum_dispatch;

pub mod constant;
pub mod sample;
pub mod upsample;
pub mod wavelength;

pub use sample::SpectralSample;
pub use wavelength::Wavelength;

pub use constant::ConstantSpectrum;
pub use upsample::{UpsampledHdrSpectrum, UpsampledSpectrum};

#[enum_dispatch]
pub trait SampleableSpectrum {
    fn evaluate_single(&self, wavelength_nm: f32) -> f32;

    fn evaluate(&self, wavelength: Wavelength) -> SpectralSample {
        SpectralSample::from_function(wavelength, |lambda| self.evaluate_single(lambda))
    }
}

#[enum_dispatch(SampleableSpectrum)]
#[derive(Debug, Clone)]
pub enum Spectrum {
    UpsampledSpectrum,
    ConstantSpectrum,
}

impl Default for Spectrum {
    fn default() -> Self {
        Self::from(ConstantSpectrum::new(0.0))
    }
}
