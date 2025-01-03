use std::simd::{cmp::SimdPartialOrd, f32x4};

use crate::math::Vec4;

#[derive(Copy, Clone)]
pub struct PdfSet {
    pub inner: Vec4,
}

impl PdfSet {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            inner: Vec4::new(x, y, z, w),
        }
        .assert_invariants()
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

    pub fn sum(self) -> f32 {
        self.inner.sum()
    }

    #[inline(always)]
    fn assert_invariants(self) -> Self {
        debug_assert!(
            self.inner.data.simd_ge(f32x4::splat(0.0)).all(),
            "PdfSet contains negative or NaN values: {:?}",
            self
        );

        self
    }
}

impl std::fmt::Debug for PdfSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entry(&self.x())
            .entry(&self.y())
            .entry(&self.z())
            .entry(&self.w())
            .finish()
    }
}

impl std::convert::From<Vec4> for PdfSet {
    fn from(inner: Vec4) -> Self {
        Self { inner }
    }
}

impl std::ops::Add<PdfSet> for PdfSet {
    type Output = PdfSet;

    fn add(self, other: Self) -> Self {
        Self {
            inner: self.inner + other.inner,
        }
        .assert_invariants()
    }
}

impl std::ops::MulAssign<PdfSet> for PdfSet {
    fn mul_assign(&mut self, other: PdfSet) {
        self.inner *= other.inner;
        self.assert_invariants();
    }
}

impl std::ops::Div<f32> for PdfSet {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self {
            inner: self.inner / other,
        }
        .assert_invariants()
    }
}

impl std::ops::Mul<PdfSet> for PdfSet {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            inner: self.inner * other.inner,
        }
        .assert_invariants()
    }
}

impl std::ops::Mul<f32> for PdfSet {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        PdfSet {
            inner: self.inner * other,
        }
        .assert_invariants()
    }
}
