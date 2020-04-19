use std::{arch::x86_64::*, mem}; // TODO: Error out in build script if AVX2 unavailable

use crate::{color::Xyz, spectrum::Wavelength};

#[derive(Copy, Clone)]
#[repr(align(16))]
pub struct SpectrumSample {
    data: __m128,
}

impl SpectrumSample {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            data: unsafe { _mm_set_ps(w, z, y, x) },
        }
        .assert_invariants()
    }

    pub fn splat(xyzw: f32) -> Self {
        Self {
            data: unsafe { _mm_set1_ps(xyzw) },
        }
        .assert_invariants()
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

    pub fn to_xyz(self, hero_wavelength: Wavelength) -> Xyz {
        let a = Xyz::from_wavelength(hero_wavelength.rotate_n(0), self.x());
        let b = Xyz::from_wavelength(hero_wavelength.rotate_n(1), self.y());
        let c = Xyz::from_wavelength(hero_wavelength.rotate_n(2), self.z());
        let d = Xyz::from_wavelength(hero_wavelength.rotate_n(3), self.w());
        a + b + c + d
    }

    pub fn from_function<F: Fn(Wavelength) -> f32>(hero_wavelength: Wavelength, func: F) -> Self {
        SpectrumSample::new(
            func(hero_wavelength.rotate_n(0)),
            func(hero_wavelength.rotate_n(1)),
            func(hero_wavelength.rotate_n(2)),
            func(hero_wavelength.rotate_n(3)),
        )
        .assert_invariants()
    }

    #[inline(always)]
    fn assert_invariants(self) -> Self {
        // Check that self.data >= 0
        debug_assert!(
            unsafe { _mm_test_all_ones(mem::transmute(_mm_cmpge_ps(self.data, _mm_setzero_ps()))) }
                == 1,
            "SpectrumSample contains negative or NaN values: {:?}",
            self
        );

        self
    }
}

impl std::fmt::Debug for SpectrumSample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpectrumSample")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .field("w", &self.w())
            .finish()
    }
}

impl std::ops::Mul<SpectrumSample> for f32 {
    type Output = SpectrumSample;

    fn mul(self, other: SpectrumSample) -> SpectrumSample {
        SpectrumSample {
            data: unsafe { _mm_mul_ps(_mm_set1_ps(self), other.data) },
        }
        .assert_invariants()
    }
}

impl std::ops::Mul<f32> for SpectrumSample {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        unsafe {
            SpectrumSample {
                data: _mm_mul_ps(self.data, _mm_set1_ps(other)),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::Mul<SpectrumSample> for SpectrumSample {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        unsafe {
            SpectrumSample {
                data: _mm_mul_ps(self.data, other.data),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::Add<SpectrumSample> for SpectrumSample {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        unsafe {
            SpectrumSample {
                data: _mm_add_ps(self.data, other.data),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::AddAssign<SpectrumSample> for SpectrumSample {
    fn add_assign(&mut self, other: SpectrumSample) {
        *self = unsafe {
            SpectrumSample {
                data: _mm_add_ps(self.data, other.data),
            }
            .assert_invariants()
        };
    }
}

impl std::ops::MulAssign<SpectrumSample> for SpectrumSample {
    fn mul_assign(&mut self, other: SpectrumSample) {
        *self = unsafe {
            SpectrumSample {
                data: _mm_mul_ps(self.data, other.data),
            }
            .assert_invariants()
        };
    }
}

impl std::ops::Div<SpectrumSample> for f32 {
    type Output = SpectrumSample;

    fn div(self, other: SpectrumSample) -> SpectrumSample {
        unsafe {
            debug_assert!(
                _mm_movemask_epi8(mem::transmute(_mm_cmpneq_ps(other.data, _mm_setzero_ps())))
                    == 0xffff,
                "division by zero: {:?}",
                self
            );

            SpectrumSample {
                data: _mm_div_ps(_mm_set1_ps(self), other.data),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::Div<f32> for SpectrumSample {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        debug_assert!(other != 0.0);

        SpectrumSample {
            data: unsafe { _mm_div_ps(self.data, _mm_set1_ps(other)) },
        }
        .assert_invariants()
    }
}

impl std::ops::DivAssign<SpectrumSample> for SpectrumSample {
    fn div_assign(&mut self, other: SpectrumSample) {
        *self = unsafe {
            SpectrumSample {
                data: _mm_div_ps(self.data, other.data),
            }
            .assert_invariants()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_div_by_zero_scalar() {
        let _ = SpectrumSample::new(1.0, 2.0, 3.0, 4.0) / 0.0;
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_div_by_zero_vector() {
        let _ = 1.0 / SpectrumSample::new(0.0, 2.0, 3.0, 4.0);
    }
}
