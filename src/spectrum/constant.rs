use crate::spectrum::{SampleableSpectrum, Wavelength};

pub struct ConstantSpectrum {
    value: f32,
}

impl ConstantSpectrum {
    #[allow(unused)]
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl SampleableSpectrum for ConstantSpectrum {
    fn evaluate_single(&self, _: Wavelength) -> f32 {
        self.value
    }
}
