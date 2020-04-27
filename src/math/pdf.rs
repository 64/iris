use std::{arch::x86_64::*, mem};

// TODO: Can we reuse code for this and SpectralSample?
#[derive(Debug, Copy, Clone)]
pub struct PdfSet {
    data: __m128,
}

impl PdfSet {
    #[allow(unused)]
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

    fn x(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.data) }
    }

    fn y(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 2, 1, 1),
            ))
        }
    }

    fn z(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 2, 1, 2),
            ))
        }
    }

    fn w(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 2, 1, 3),
            ))
        }
    }

    pub fn hero(self) -> f32 {
        self.x()
    }

    pub fn sum(self) -> f32 {
        // Can be optimized further
        self.x() + self.y() + self.z() + self.w()
    }

    #[allow(unused)]
    pub fn is_zero(self) -> bool {
        // Can be optimized further
        self.x() == 0.0 && self.y() == 0.0 && self.z() == 0.0 && self.w() == 0.0
    }

    #[inline(always)]
    fn assert_invariants(self) -> Self {
        debug_assert!(
            unsafe { _mm_test_all_ones(mem::transmute(_mm_cmpge_ps(self.data, _mm_setzero_ps()))) }
                == 1,
            "PdfSet contains negative or NaN values: {:?}",
            self
        );

        self
    }
}

impl std::ops::Add<PdfSet> for PdfSet {
    type Output = PdfSet;

    fn add(self, other: Self) -> Self {
        unsafe {
            Self {
                data: _mm_add_ps(self.data, other.data),
            }
            .assert_invariants()
        }
    }
}
