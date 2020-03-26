#![allow(dead_code)]

use super::Global;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct Vec3<System = Global> {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _coord: PhantomData<System>,
}

impl<S> Vec3<S> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            _coord: PhantomData,
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn len_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn len(self) -> f32 {
        self.len_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
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
    }
}

impl<S> std::ops::Mul for Vec3<S> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl<S> std::ops::MulAssign for Vec3<S> {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
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
    }
}

impl<S> std::ops::Neg for Vec3<S> {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}
