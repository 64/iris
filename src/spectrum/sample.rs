use std::{arch::x86_64::*, mem};

use crate::{color::Xyz, spectrum::Wavelength};

#[derive(Copy, Clone)]
pub struct SpectralSample {
    data: __m128,
}

impl SpectralSample {
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

    pub fn to_xyz(self, wavelength: Wavelength) -> Xyz {
        // TODO: Simd
        let a = Xyz::from_wavelength(wavelength.x(), self.x());
        let b = Xyz::from_wavelength(wavelength.y(), self.y());
        let c = Xyz::from_wavelength(wavelength.z(), self.z());
        let d = Xyz::from_wavelength(wavelength.w(), self.w());
        a + b + c + d
    }

    pub fn from_function<F: Fn(f32) -> f32>(wavelength: Wavelength, func: F) -> Self {
        SpectralSample::new(
            func(wavelength.x()),
            func(wavelength.y()),
            func(wavelength.z()),
            func(wavelength.w()),
        )
        .assert_invariants()
    }

    #[inline(always)]
    fn assert_invariants(self) -> Self {
        // Check that self.data >= 0
        debug_assert!(
            unsafe { _mm_test_all_ones(mem::transmute(_mm_cmpge_ps(self.data, _mm_setzero_ps()))) }
                == 1,
            "SpectralSample contains negative or NaN values: {:?}",
            self
        );

        self
    }

    pub fn clamp(self, min: f32, max: f32) -> Self {
        Self {
            data: unsafe { _mm_min_ps(_mm_max_ps(self.data, _mm_set1_ps(min)), _mm_set1_ps(max)) },
        }
        .assert_invariants()
    }

    pub fn sum(self) -> f32 {
        // Can be optimized further
        self.x() + self.y() + self.z() + self.w()
    }

    pub fn is_zero(self) -> bool {
        // Can be optimized further
        self.x() == 0.0 && self.y() == 0.0 && self.z() == 0.0 && self.w() == 0.0
    }
}

impl std::fmt::Debug for SpectralSample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpectralSample")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .field("w", &self.w())
            .finish()
    }
}

impl std::ops::Mul<SpectralSample> for f32 {
    type Output = SpectralSample;

    fn mul(self, other: SpectralSample) -> SpectralSample {
        SpectralSample {
            data: unsafe { _mm_mul_ps(_mm_set1_ps(self), other.data) },
        }
        .assert_invariants()
    }
}

impl std::ops::Mul<f32> for SpectralSample {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        unsafe {
            SpectralSample {
                data: _mm_mul_ps(self.data, _mm_set1_ps(other)),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::Mul<SpectralSample> for SpectralSample {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        unsafe {
            SpectralSample {
                data: _mm_mul_ps(self.data, other.data),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::Add<SpectralSample> for SpectralSample {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        unsafe {
            SpectralSample {
                data: _mm_add_ps(self.data, other.data),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::AddAssign<SpectralSample> for SpectralSample {
    fn add_assign(&mut self, other: SpectralSample) {
        *self = unsafe {
            SpectralSample {
                data: _mm_add_ps(self.data, other.data),
            }
            .assert_invariants()
        };
    }
}

impl std::ops::MulAssign<SpectralSample> for SpectralSample {
    fn mul_assign(&mut self, other: SpectralSample) {
        *self = unsafe {
            SpectralSample {
                data: _mm_mul_ps(self.data, other.data),
            }
            .assert_invariants()
        };
    }
}

impl std::ops::Div<SpectralSample> for f32 {
    type Output = SpectralSample;

    fn div(self, other: SpectralSample) -> SpectralSample {
        unsafe {
            debug_assert!(
                _mm_movemask_epi8(mem::transmute(_mm_cmpneq_ps(other.data, _mm_setzero_ps())))
                    == 0xffff,
                "division by zero: {:?}",
                self
            );

            SpectralSample {
                data: _mm_div_ps(_mm_set1_ps(self), other.data),
            }
            .assert_invariants()
        }
    }
}

impl std::ops::Div<f32> for SpectralSample {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        debug_assert!(other != 0.0);

        SpectralSample {
            data: unsafe { _mm_div_ps(self.data, _mm_set1_ps(other)) },
        }
        .assert_invariants()
    }
}

impl std::ops::DivAssign<SpectralSample> for SpectralSample {
    fn div_assign(&mut self, other: SpectralSample) {
        *self = unsafe {
            SpectralSample {
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
        let _ = SpectralSample::new(1.0, 2.0, 3.0, 4.0) / 0.0;
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_div_by_zero_vector() {
        let _ = 1.0 / SpectralSample::new(0.0, 2.0, 3.0, 4.0);
    }
}
