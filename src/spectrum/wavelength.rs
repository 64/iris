use crate::sampling::Sampler;
use std::arch::x86_64::*;

pub const LAMBDA_MIN_NM: f32 = 360.0;
pub const LAMBDA_MAX_NM: f32 = 830.0;
pub const LAMBDA_RANGE_NM: f32 = LAMBDA_MAX_NM - LAMBDA_MIN_NM;

#[derive(Debug, Copy, Clone)]
pub struct Wavelength {
    data: __m128,
}

impl Wavelength {
    pub fn new(hero: f32) -> Self {
        let y = rotate_n(hero, 1);
        let z = rotate_n(hero, 2);
        let w = rotate_n(hero, 3);

        Self {
            data: unsafe { _mm_set_ps(w, z, y, hero) },
        }
    }

    // TODO: We should use golden ratio sampling, I think
    pub fn sample(sampler: &mut Sampler) -> Self {
        Self::new(sampler.gen_range(LAMBDA_MIN_NM, LAMBDA_MAX_NM))
    }

    pub fn pdf(self) -> f32 {
        1.0 / LAMBDA_RANGE_NM
    }

    pub fn hero(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.data) }
    }

    pub fn x(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.data) }
    }

    pub fn y(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 2, 1, 1),
            ))
        }
    }

    pub fn z(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 2, 1, 2),
            ))
        }
    }

    pub fn w(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 2, 1, 3),
            ))
        }
    }
}

pub fn rotate_n(hero: f32, n: usize) -> f32 {
    let lambda = hero + (LAMBDA_RANGE_NM / 4.0) * (n as f32);

    // Perform modulo operation (so that lambda is always in range)
    if lambda >= LAMBDA_MAX_NM {
        lambda - LAMBDA_RANGE_NM
    } else {
        lambda
    }
}

// Importance sample the wavelength with pdf(lambda) = sech^2(0.0072(lambda -
// 538)) See https://www.researchgate.net/publication/228938842_An_Improved_Technique_for_Full_Spectral_Rendering
// Even though we stratify the spectrum for HWSS, it's important that the hero
// wavelength is importance sampled too because we use it for path
// generation.

// TODO: Is the error on these curves reasonable?
// fn pdf(lambda: f32) -> f32 {
// if lambda < 380.0 || lambda > 700.0 {
// 0.0
//} else {
// let x1 = lambda;
// let x2 = lambda * lambda;
// let x3 = lambda * x2;
// let x4 = x2 * x2;
// let x5 = lambda * x4;
//-8.19329974e-16 
//-8.19329974e-16 * x5 + 5.58900125e-12 * x4 - 9.63692860e-09 * x3 +
//-8.19329974e-16   6.92631892e-06 * x2
//- 2.22283548e-03 * x1
//+ 2.64835297e-01
//}

// fn inverse_cdf(unif: f32) -> f32 {
// let val = 377.92772964 * unif.powi(3) - 562.7179108 * unif.powi(2)
//+ 495.09783553 * unif
//+ 384.47036553;
// val.clamp(LAMBDA_MIN_NM, LAMBDA_MAX_NM)
//}
