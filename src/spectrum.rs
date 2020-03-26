use crate::color::Xyz;

pub const LAMBDA_MIN_NM: f32 = 380.0;
pub const LAMBDA_MAX_NM: f32 = 700.0;
pub const LAMBDA_RANGE_NM: f32 = LAMBDA_MAX_NM - LAMBDA_MIN_NM;

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

    pub fn to_xyz(self, hero_wavelength: f32) -> Xyz {
        let a = Xyz::from_wavelength(lambda_n(hero_wavelength, 0), self.x);
        let b = Xyz::from_wavelength(lambda_n(hero_wavelength, 1), self.y);
        let c = Xyz::from_wavelength(lambda_n(hero_wavelength, 2), self.z);
        let d = Xyz::from_wavelength(lambda_n(hero_wavelength, 3), self.w);
        a + b + c + d
    }

    pub fn map<F: Fn(f32, f32) -> f32>(self, hero_wavelength: f32, func: F) -> Self {
        SpectrumSample::new(
            func(self.x, lambda_n(hero_wavelength, 0)),
            func(self.y, lambda_n(hero_wavelength, 1)),
            func(self.z, lambda_n(hero_wavelength, 2)),
            func(self.w, lambda_n(hero_wavelength, 3)),
        )
    }
}

pub fn lambda_n(hero_wavelength: f32, n: usize) -> f32 {
    let lambda = hero_wavelength + LAMBDA_RANGE_NM * (n as f32);

    // Perform modulo operation (so that lambda is always in range)
    if lambda >= LAMBDA_MAX_NM {
        lambda - LAMBDA_RANGE_NM
    } else {
        lambda
    }
}
