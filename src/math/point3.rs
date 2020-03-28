#![allow(dead_code)]

use super::{Global, Vec3};
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct Point3<System = Global> {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _coord: PhantomData<System>,
}

impl<S> Point3<S> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            _coord: PhantomData,
        }
    }

    pub fn distance(self, other: Self) -> f32 {
        self.distance_squared(other).sqrt()
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).len_squared()
    }

    pub fn to_vec(self) -> Vec3<S> {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn to_nalgebra(self) -> bvh::nalgebra::Point3<f32> {
        bvh::nalgebra::Point3::new(self.x, self.y, self.z)
    }
}

// Required because #[derive(Copy, Clone)] places bounds on type parameters
impl<S> Copy for Point3<S> {}
impl<S> Clone for Point3<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> std::cmp::PartialEq<f32> for Point3<S> {
    fn eq(&self, other: &f32) -> bool {
        self.x == *other && self.y == *other && self.z == *other
    }
}

impl<S> std::ops::Sub for Point3<S> {
    type Output = Vec3<S>;

    fn sub(self, other: Self) -> Vec3<S> {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<S> std::ops::Add<Vec3<S>> for Point3<S> {
    type Output = Self;

    fn add(self, other: Vec3<S>) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<S> std::ops::AddAssign<Vec3<S>> for Point3<S> {
    fn add_assign(&mut self, other: Vec3<S>) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl<S> std::ops::Sub<Vec3<S>> for Point3<S> {
    type Output = Self;

    fn sub(self, other: Vec3<S>) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<S> From<Vec3<S>> for Point3<S> {
    fn from(v: Vec3<S>) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}
