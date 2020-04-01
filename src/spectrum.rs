use crate::{
    color::Xyz,
    sampler::Sampler,
};

pub const LAMBDA_MIN_NM: f32 = 380.0;
pub const LAMBDA_MAX_NM: f32 = 700.0;
pub const LAMBDA_RANGE_NM: f32 = LAMBDA_MAX_NM - LAMBDA_MIN_NM;

#[derive(Debug, Copy, Clone)]
pub struct Wavelength(f32);

impl Wavelength {
    pub fn as_nm_f32(self) -> f32 {
        self.0
    }


    pub fn rotate_n(self, n: usize) -> Wavelength {
        let lambda = self.0 + (LAMBDA_RANGE_NM / 4.0) * (n as f32);

        // Perform modulo operation (so that lambda is always in range)
        if lambda >= LAMBDA_MAX_NM {
            Self(lambda - LAMBDA_RANGE_NM)
        } else {
            Self(lambda)
        }
    }

    // Uniform sampling
    pub fn sample(sampler: &mut Sampler) -> Self {
        Wavelength(sampler.gen_range(LAMBDA_MIN_NM, LAMBDA_MAX_NM))
    }

    pub fn pdf(self) -> f32 {
        1.0 / LAMBDA_RANGE_NM
    }
}

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

    pub fn from_spectrum<F: Fn(Wavelength) -> f32>(hero_wavelength: Wavelength, func: F) -> Self {
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
        SpectrumSample::new(self * other.x, self * other.y, self * other.z, self * other.w)
    }
}

impl std::ops::Mul<f32> for SpectrumSample {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self::new(self.x * other, self.y * other, self.z * other, self.w * other)
    }
}
