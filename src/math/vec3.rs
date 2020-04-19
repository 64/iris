#![allow(dead_code)]

use super::World;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Vec3<System = World> {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _coord: PhantomData<System>,
}

impl<S> Vec3<S> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let ret = Self {
            x,
            y,
            z,
            _coord: PhantomData,
        };

        ret.assert_invariants();

        ret
    }

    fn assert_invariants(&self) {
        debug_assert!(self.x.is_finite());
        debug_assert!(self.y.is_finite());
        debug_assert!(self.z.is_finite());
    }

    pub fn splat(x: f32) -> Self {
        Self::new(x, x, x)
    }

    pub fn x(self) -> f32 {
        self.x
    }

    pub fn y(self) -> f32 {
        self.y
    }

    pub fn z(self) -> f32 {
        self.z
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Vec3::new(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }

    pub fn len_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn len(self) -> f32 {
        self.len_squared().sqrt()
    }

    pub fn normalize(self) -> Self {
        self / self.len()
    }
}

// Required because #[derive(Copy, Clone)] places bounds on type parameters
impl<S> Copy for Vec3<S> {}
impl<S> Clone for Vec3<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> std::cmp::PartialEq<f32> for Vec3<S> {
    fn eq(&self, other: &f32) -> bool {
        self.x == *other && self.y == *other && self.z == *other
    }
}

impl<S> std::cmp::PartialEq<Vec3<S>> for Vec3<S> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl<S> std::ops::Add for Vec3<S> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<S> std::ops::AddAssign for Vec3<S> {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.assert_invariants();
    }
}

impl<S> std::ops::Add<f32> for Vec3<S> {
    type Output = Self;

    fn add(self, other: f32) -> Self {
        Self::new(self.x + other, self.y + other, self.z + other)
    }
}

impl<S> std::ops::AddAssign<f32> for Vec3<S> {
    fn add_assign(&mut self, other: f32) {
        self.x += other;
        self.y += other;
        self.z += other;
        self.assert_invariants();
    }
}

impl<S> std::ops::Sub for Vec3<S> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<S> std::ops::SubAssign for Vec3<S> {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.assert_invariants();
    }
}

impl<S> std::ops::Sub<f32> for Vec3<S> {
    type Output = Self;

    fn sub(self, other: f32) -> Self {
        Self::new(self.x - other, self.y - other, self.z - other)
    }
}

impl<S> std::ops::SubAssign<f32> for Vec3<S> {
    fn sub_assign(&mut self, other: f32) {
        self.x -= other;
        self.y -= other;
        self.z -= other;
        self.assert_invariants();
    }
}

impl<S> std::ops::Mul<Vec3<S>> for f32 {
    type Output = Vec3<S>;

    fn mul(self, other: Vec3<S>) -> Vec3<S> {
        Vec3::new(self * other.x, self * other.y, self * other.z)
    }
}

impl<S> std::ops::Mul<f32> for Vec3<S> {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        Self::new(self.x * other, self.y * other, self.z * other)
    }
}

impl<S> std::ops::MulAssign<f32> for Vec3<S> {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
        self.assert_invariants();
    }
}

impl<S> std::ops::Div for Vec3<S> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }
}

impl<S> std::ops::DivAssign for Vec3<S> {
    fn div_assign(&mut self, other: Self) {
        self.x /= other.x;
        self.y /= other.y;
        self.z /= other.z;
        self.assert_invariants();
    }
}

impl<S> std::ops::Div<f32> for Vec3<S> {
    type Output = Self;

    fn div(self, other: f32) -> Self {
        Self::new(self.x / other, self.y / other, self.z / other)
    }
}

impl<S> std::ops::DivAssign<f32> for Vec3<S> {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
        self.assert_invariants();
    }
}

impl<S> std::ops::Neg for Vec3<S> {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}
