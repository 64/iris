use std::{arch::x86_64::*, mem};

use crate::{color::Xyz, spectrum::Wavelength, math::Vec4};

#[derive(Copy, Clone)]
pub struct SpectralSample {
    pub inner: Vec4,
}

impl SpectralSample {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            inner: Vec4::new(x, y, z, w),
        }.assert_invariants()
    }

    pub fn splat(xyzw: f32) -> Self {
        Self {
            inner: Vec4::splat(xyzw),
        }
        .assert_invariants()
    }

    pub fn hero(self) -> f32 {
        self.inner.x()
    }

    pub fn x(self) -> f32 {
        self.inner.x()
    }

    pub fn y(self) -> f32 {
        self.inner.y()
    }

    pub fn z(self) -> f32 {
        self.inner.z()
    }

    pub fn w(self) -> f32 {
        self.inner.w()
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
            unsafe { _mm_test_all_ones(mem::transmute(_mm_cmpge_ps(self.inner.data, _mm_setzero_ps()))) }
                == 1,
            "SpectralSample contains negative or NaN values: {:?}",
            self
        );

        self
    }

    pub fn clamp(self, min: f32, max: f32) -> Self {
        Self {
            inner: self.inner.clamp(min, max)
        }
    }

    pub fn sum(self) -> f32 {
        self.inner.sum()
    }

    pub fn is_zero(self) -> bool {
        self.inner.is_zero()
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

impl std::convert::From<Vec4> for SpectralSample {
    fn from(inner: Vec4) -> Self {
        Self { inner }
    }
}

impl std::ops::Mul<SpectralSample> for f32 {
    type Output = SpectralSample;

    fn mul(self, other: SpectralSample) -> SpectralSample {
        SpectralSample {
            inner: self * other.inner,
        }
        .assert_invariants()
    }
}

impl std::ops::Mul<f32> for SpectralSample {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        SpectralSample {
            inner: self.inner * other,
        }
        .assert_invariants()
    }
}

impl std::ops::Mul<SpectralSample> for SpectralSample {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            inner: self.inner * other.inner,
        }
        .assert_invariants()
    }
}

impl std::ops::Add<SpectralSample> for SpectralSample {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SpectralSample {
            inner: self.inner + other.inner,
        }
        .assert_invariants()
    }
}

impl std::ops::AddAssign<SpectralSample> for SpectralSample {
    fn add_assign(&mut self, other: SpectralSample) {
        self.inner += other.inner;
        self.assert_invariants();
    }
}

impl std::ops::MulAssign<SpectralSample> for SpectralSample {
    fn mul_assign(&mut self, other: SpectralSample) {
        self.inner *= other.inner;
        self.assert_invariants();
    }
}

impl std::ops::Div<SpectralSample> for f32 {
    type Output = SpectralSample;

    fn div(self, other: SpectralSample) -> SpectralSample {
        SpectralSample {
            inner: self / other.inner
        }
        .assert_invariants()
    }
}

impl std::ops::Div<f32> for SpectralSample {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            inner: self.inner / other,
        }
        .assert_invariants()
    }
}

impl std::ops::DivAssign<SpectralSample> for SpectralSample {
    fn div_assign(&mut self, other: SpectralSample) {
        self.inner /= other.inner;
        self.assert_invariants();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_nans() {
        let _ = SpectralSample::new(1.0, f32::NAN, 1.0, 1.0);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_negatives() {
        let _ = SpectralSample::new(1.0, -1.0, 1.0, 1.0);
    }
}
