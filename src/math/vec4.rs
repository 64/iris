use std::arch::x86_64::*;
use std::mem;

// TODO: Fused multiply-add etc, rsqrt, benchmarks, fallback...

// See http://www.codersnotes.com/notes/maths-lib-2016/

#[derive(Copy, Clone)]
pub struct Vec4 {
    pub data: __m128,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            data: unsafe { _mm_set_ps(w, z, y, x) },
        }
    }

    pub fn splat(xyzw: f32) -> Self {
        Self {
            data: unsafe { _mm_set1_ps(xyzw) },
        }
    }

    pub fn x(self) -> f32 {
        unsafe { _mm_cvtss_f32(self.data) }
    }

    pub fn y(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(1, 1, 1, 1),
            ))
        }
    }

    pub fn z(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(2, 2, 2, 2),
            ))
        }
    }

    pub fn w(self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_shuffle_ps(
                self.data,
                self.data,
                _MM_SHUFFLE(3, 3, 3, 3),
            ))
        }
    }

    pub fn clamp(self, min: f32, max: f32) -> Self {
        Self {
            data: unsafe { _mm_min_ps(_mm_max_ps(self.data, _mm_set1_ps(min)), _mm_set1_ps(max)) },
        }
    }

    // TODO: Is this the quickest way?
    // https://stackoverflow.com/questions/4120681/how-to-calculate-single-vector-dot-product-using-sse-intrinsic-functions-in-c
    pub fn dot(self, other: Self) -> f32 {
        unsafe {
            _mm_cvtss_f32(_mm_dp_ps(self.data, other.data, 0xff))
        }
    }

    pub fn sum(self) -> f32 {
        // Can be optimized further
        unsafe {
            _mm_cvtss_f32(_mm_dp_ps(self.data, _mm_set1_ps(1.0), 0xff))
        }
    }

    pub fn is_zero(self) -> bool {
        // Can be optimized further
        unsafe {
            _mm_movemask_ps(
                _mm_cmpeq_ps(self.data, _mm_setzero_ps())
            ) == 0xf
        }
    }
}

impl std::fmt::Debug for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vec4")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .field("w", &self.w())
            .finish()
    }
}

impl std::ops::Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Vec4 {
        Vec4 {
            data: unsafe { _mm_mul_ps(_mm_set1_ps(self), other.data) },
        }
    }
}

impl std::ops::Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        unsafe {
            Vec4 {
                data: _mm_mul_ps(self.data, _mm_set1_ps(other)),
            }
        }
    }
}

impl std::ops::Mul<Vec4> for Vec4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        unsafe {
            Vec4 {
                data: _mm_mul_ps(self.data, other.data),
            }
        }
    }
}

impl std::ops::Add<Vec4> for Vec4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        unsafe {
            Vec4 {
                data: _mm_add_ps(self.data, other.data),
            }
        }
    }
}

impl std::ops::Add<f32> for Vec4 {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        unsafe {
            Vec4 {
                data: _mm_add_ps(self.data, _mm_set1_ps(other)),
            }
        }
    }
}

impl std::ops::AddAssign<Vec4> for Vec4 {
    fn add_assign(&mut self, other: Vec4) {
        *self = unsafe {
            Vec4 {
                data: _mm_add_ps(self.data, other.data),
            }
        };
    }
}

impl std::ops::AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, other: f32) {
        *self = unsafe {
            Vec4 {
                data: _mm_add_ps(self.data, _mm_set1_ps(other)),
            }
        };
    }
}

impl std::ops::MulAssign<Vec4> for Vec4 {
    fn mul_assign(&mut self, other: Vec4) {
        *self = unsafe {
            Vec4 {
                data: _mm_mul_ps(self.data, other.data),
            }
        };
    }
}

impl std::ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, other: f32) {
        *self = unsafe {
            Vec4 {
                data: _mm_mul_ps(self.data, _mm_set1_ps(other)),
            }
        };
    }
}

impl std::ops::Sub for Vec4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        unsafe {
            Vec4 {
                data: _mm_sub_ps(self.data, other.data),
            }
        }
    }
}

impl std::ops::Sub<f32> for Vec4 {
    type Output = Self;

    fn sub(self, other: f32) -> Self {
        unsafe {
            Vec4 {
                data: _mm_sub_ps(self.data, _mm_set1_ps(other)),
            }
        }
    }
}

impl std::ops::SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Vec4) {
        *self = unsafe {
            Vec4 {
                data: _mm_sub_ps(self.data, other.data),
            }
        };
    }
}

impl std::ops::SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, other: f32) {
        *self = unsafe {
            Vec4 {
                data: _mm_sub_ps(self.data, _mm_set1_ps(other)),
            }
        };
    }
}

impl std::ops::Div<Vec4> for f32 {
    type Output = Vec4;

    fn div(self, other: Vec4) -> Vec4 {
        unsafe {
            debug_assert!(
                _mm_movemask_ps(mem::transmute(_mm_cmpeq_ps(other.data, _mm_setzero_ps())))
                    == 0x0,
                "division by zero: {:?}",
                self
            );

            Vec4 {
                data: _mm_div_ps(_mm_set1_ps(self), other.data),
            }
        }
    }
}

impl std::ops::Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        debug_assert!(other != 0.0);

        Vec4 {
            data: unsafe { _mm_div_ps(self.data, _mm_set1_ps(other)) },
        }
    }
}

impl std::ops::Div<Vec4> for Vec4 {
    type Output = Vec4;

    fn div(self, other: Vec4) -> Vec4 {
        unsafe {
            debug_assert!(
                _mm_movemask_ps(mem::transmute(_mm_cmpeq_ps(other.data, _mm_setzero_ps())))
                    == 0x0,
                "division by zero: {:?}",
                self
            );

            Vec4 {
                data: _mm_div_ps(self.data, other.data),
            }
        }
    }
}

impl std::ops::DivAssign<Vec4> for Vec4 {
    fn div_assign(&mut self, other: Vec4) {
        *self = unsafe {
            Vec4 {
                data: _mm_div_ps(self.data, other.data),
            }
        };
    }
}

impl std::ops::DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, other: f32) {
        *self = unsafe {
            Vec4 {
                data: _mm_div_ps(self.data, _mm_set1_ps(other)),
            }
        };
    }
}

impl std::ops::Neg for Vec4 {
    type Output = Self;

    fn neg(self) -> Self {
        unsafe {
            Vec4 {
                data: _mm_sub_ps(_mm_setzero_ps(), self.data)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert!(Vec4::splat(0.0).is_zero());
    }

    #[test]
    fn test_dot() {
        assert_eq!(Vec4::new(1.0, 2.0, 3.0, 4.0).dot(Vec4::new(4.0, 3.0, 2.0, 1.0)), 20.0);
    }

    #[test]
    fn test_sum() {
        assert_eq!(Vec4::new(1.0, 2.0, 3.0, 4.0).sum(), 10.0);
        assert_eq!(Vec4::new(2.0, 1.0, 4.0, 3.0).sum(), 10.0);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_div_by_zero_vector() {
        let _ = 1.0 / Vec4::new(1.0, 0.0, 1.0, 1.0);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn test_div_by_zero_scalar() {
        let _ = Vec4::splat(1.0) / 0.0;
    }
}
