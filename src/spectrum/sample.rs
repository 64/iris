use crate::{color::Xyz, spectrum::Wavelength};

#[derive(Debug, Copy, Clone)]
pub struct SpectrumSample {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl SpectrumSample {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn splat(xyzw: f32) -> Self {
        Self::new(xyzw, xyzw, xyzw, xyzw)
    }

    pub fn to_xyz(self, hero_wavelength: Wavelength) -> Xyz {
        let a = Xyz::from_wavelength(hero_wavelength.rotate_n(0), self.x);
        let b = Xyz::from_wavelength(hero_wavelength.rotate_n(1), self.y);
        let c = Xyz::from_wavelength(hero_wavelength.rotate_n(2), self.z);
        let d = Xyz::from_wavelength(hero_wavelength.rotate_n(3), self.w);
        a + b + c + d
    }

    pub fn from_function<F: Fn(Wavelength) -> f32>(hero_wavelength: Wavelength, func: F) -> Self {
        SpectrumSample::new(
            func(hero_wavelength.rotate_n(0)),
            func(hero_wavelength.rotate_n(1)),
            func(hero_wavelength.rotate_n(2)),
            func(hero_wavelength.rotate_n(3)),
        )
    }
}

impl std::ops::Mul<SpectrumSample> for f32 {
    type Output = SpectrumSample;

    fn mul(self, other: SpectrumSample) -> SpectrumSample {
        SpectrumSample::new(
            self * other.x,
            self * other.y,
            self * other.z,
            self * other.w,
        )
    }
}

impl std::ops::Mul<f32> for SpectrumSample {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self::new(
            self.x * other,
            self.y * other,
            self.z * other,
            self.w * other,
        )
    }
}

impl std::ops::Div<SpectrumSample> for f32 {
    type Output = SpectrumSample;

    fn div(self, other: SpectrumSample) -> SpectrumSample {
        SpectrumSample::new(
            self / other.x,
            self / other.y,
            self / other.z,
            self / other.w,
        )
    }
}

impl std::ops::Div<f32> for SpectrumSample {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self::new(
            self.x / other,
            self.y / other,
            self.z / other,
            self.w / other,
        )
    }
}
