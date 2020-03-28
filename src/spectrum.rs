use crate::{
    color::Xyz,
    sampler::{Sampleable, Sampler},
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
}

impl Sampleable for Wavelength {
    // Uniform sampling
    fn sample(sampler: &mut Sampler) -> Self {
        Wavelength(sampler.gen_range(LAMBDA_MIN_NM, LAMBDA_MAX_NM))
    }

    fn pdf(_: &Self) -> f32 {
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
        let a = Xyz::from_wavelength(lambda_n(hero_wavelength, 0), self.x);
        let b = Xyz::from_wavelength(lambda_n(hero_wavelength, 1), self.y);
        let c = Xyz::from_wavelength(lambda_n(hero_wavelength, 2), self.z);
        let d = Xyz::from_wavelength(lambda_n(hero_wavelength, 3), self.w);
        (a + b + c + d) * 0.25
    }

    pub fn map<F: Fn(f32, Wavelength) -> f32>(self, hero_wavelength: Wavelength, func: F) -> Self {
        SpectrumSample::new(
            func(self.x, lambda_n(hero_wavelength, 0)),
            func(self.y, lambda_n(hero_wavelength, 1)),
            func(self.z, lambda_n(hero_wavelength, 2)),
            func(self.w, lambda_n(hero_wavelength, 3)),
        )
    }
}

pub fn lambda_n(hero_wavelength: Wavelength, n: usize) -> Wavelength {
    let lambda = hero_wavelength.0 + (LAMBDA_RANGE_NM / 4.0) * (n as f32);

    // Perform modulo operation (so that lambda is always in range)
    if lambda >= LAMBDA_MAX_NM {
        Wavelength(lambda - LAMBDA_RANGE_NM)
    } else {
        Wavelength(lambda)
    }
}
