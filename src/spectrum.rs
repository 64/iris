use crate::color::Xyz;

pub const LAMBDA_MIN_NM: f32 = 380.0;
pub const LAMBDA_MAX_NM: f32 = 700.0;

#[derive(Debug, Copy, Clone)]
pub struct SpectrumSample {
    wavelength: f32,
    value: f32,
}

impl SpectrumSample {
    pub fn new(wavelength: f32, value: f32) -> Self {
        Self { wavelength, value }
    }

    pub fn to_xyz(self) -> Xyz {
        Xyz::from_wavelength(self.wavelength, self.value)
    }
}
