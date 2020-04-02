use enum_dispatch::enum_dispatch;

pub mod constant;
pub mod sample;
pub mod upsample;
pub mod wavelength;

pub use sample::SpectrumSample;
pub use wavelength::Wavelength;

pub use constant::ConstantSpectrum;
pub use upsample::UpsampledSpectrum;

#[enum_dispatch]
pub trait SampleableSpectrum {
    fn evaluate_single(&self, wavelength: Wavelength) -> f32;

    fn evaluate(&self, hero_wavelength: Wavelength) -> SpectrumSample {
        SpectrumSample::from_function(hero_wavelength, |lambda| self.evaluate_single(lambda))
    }
}

#[enum_dispatch(SampleableSpectrum)]
pub enum Spectrum {
    UpsampledSpectrum,
    ConstantSpectrum,
}
