use std::simd::{cmp::SimdPartialEq, f32x4, num::SimdFloat};

// TODO: Fused multiply-add etc, rsqrt, benchmarks, fallback...

// See http://www.codersnotes.com/notes/maths-lib-2016/

#[derive(Copy, Clone)]
pub struct Vec4 {
    pub data: f32x4,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            data: f32x4::from_array([x, y, z, w]),
        }
    }

    pub fn splat(xyzw: f32) -> Self {
        Self {
            data: f32x4::splat(xyzw),
        }
    }

    pub fn hero(self) -> f32 {
        self.x()
    }

    pub fn x(self) -> f32 {
        self.data[0]
    }

    pub fn y(self) -> f32 {
        self.data[1]
    }

    pub fn z(self) -> f32 {
        self.data[2]
    }

    pub fn w(self) -> f32 {
        self.data[3]
    }

    pub fn clamp(self, min: f32, max: f32) -> Self {
        Self {
            data: self.data.simd_clamp(f32x4::splat(min), f32x4::splat(max)),
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        (self.data * other.data).reduce_sum()
    }

    pub fn sum(self) -> f32 {
        self.data.reduce_sum()
    }

    pub fn is_zero(self) -> bool {
        self.data == f32x4::splat(0.0)
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
            data: f32x4::splat(self) * other.data,
        }
    }
}

impl std::ops::Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Vec4 {
            data: self.data * f32x4::splat(other),
        }
    }
}

impl std::ops::Mul<Vec4> for Vec4 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Vec4 {
            data: self.data * other.data,
        }
    }
}

impl std::ops::Add<Vec4> for Vec4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Vec4 {
            data: self.data + other.data,
        }
    }
}

impl std::ops::Add<f32> for Vec4 {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        Vec4 {
            data: self.data + f32x4::splat(other),
        }
    }
}

impl std::ops::Add<Vec4> for f32 {
    type Output = Vec4;

    fn add(self, other: Vec4) -> Vec4 {
        Vec4 {
            data: f32x4::splat(self) + other.data,
        }
    }
}

impl std::ops::AddAssign<Vec4> for Vec4 {
    fn add_assign(&mut self, other: Vec4) {
        *self = Vec4 {
            data: self.data + other.data,
        };
    }
}

impl std::ops::AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, other: f32) {
        *self = Vec4 {
            data: self.data + f32x4::splat(other),
        };
    }
}

impl std::ops::MulAssign<Vec4> for Vec4 {
    fn mul_assign(&mut self, other: Vec4) {
        *self = Vec4 {
            data: self.data * other.data,
        };
    }
}

impl std::ops::MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, other: f32) {
        *self = Vec4 {
            data: self.data * f32x4::splat(other),
        };
    }
}

impl std::ops::Sub for Vec4 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Vec4 {
            data: self.data - other.data,
        }
    }
}

impl std::ops::Sub<f32> for Vec4 {
    type Output = Self;

    fn sub(self, other: f32) -> Self {
        Vec4 {
            data: self.data - f32x4::splat(other),
        }
    }
}

impl std::ops::SubAssign for Vec4 {
    fn sub_assign(&mut self, other: Vec4) {
        *self = Vec4 {
            data: self.data - other.data,
        };
    }
}

impl std::ops::SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, other: f32) {
        *self = Vec4 {
            data: self.data - f32x4::splat(other),
        };
    }
}

impl std::ops::Div<Vec4> for f32 {
    type Output = Vec4;

    fn div(self, other: Vec4) -> Vec4 {
        debug_assert!(
            !other.data.simd_eq(f32x4::splat(0.0)).any(),
            "division by zero: {:?}",
            other
        );
        Vec4 {
            data: f32x4::splat(self) / other.data,
        }
    }
}

impl std::ops::Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        debug_assert_ne!(other, 0.0, "division by zero");
        Vec4 {
            data: self.data / f32x4::splat(other),
        }
    }
}

impl std::ops::Div<Vec4> for Vec4 {
    type Output = Vec4;

    fn div(self, other: Vec4) -> Vec4 {
        debug_assert!(
            !other.data.simd_eq(f32x4::splat(0.0)).any(),
            "division by zero: {:?}",
            other
        );
        Vec4 {
            data: self.data / other.data,
        }
    }
}

impl std::ops::DivAssign<Vec4> for Vec4 {
    fn div_assign(&mut self, other: Vec4) {
        debug_assert!(
            !other.data.simd_eq(f32x4::splat(0.0)).any(),
            "division by zero: {:?}",
            other
        );
        *self = Vec4 {
            data: self.data / other.data,
        };
    }
}

impl std::ops::DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, other: f32) {
        debug_assert_ne!(other, 0.0, "division by zero");
        *self = Vec4 {
            data: self.data / f32x4::splat(other),
        };
    }
}

impl std::ops::Neg for Vec4 {
    type Output = Self;

    fn neg(self) -> Self {
        Vec4 { data: -self.data }
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
    fn test_neg() {
        assert!((-Vec4::splat(1.0) - Vec4::splat(-1.0)).is_zero());
    }

    #[test]
    fn test_dot() {
        assert_eq!(
            Vec4::new(1.0, 2.0, 3.0, 4.0).dot(Vec4::new(4.0, 3.0, 2.0, 1.0)),
            20.0
        );
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
