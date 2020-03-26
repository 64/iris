#![allow(dead_code)]

use super::{Global, Point3, Vec3};

#[derive(Debug, Clone)]
pub struct Ray<System = Global> {
    pub o: Point3<System>,
    pub d: Vec3<System>,
    pub t_max: f32,
}

impl<S> Ray<S> {
    pub fn new(o: Point3<S>, d: Vec3<S>) -> Self {
        Self {
            o,
            d,
            t_max: std::f32::INFINITY,
        }
    }

    pub fn new_t_max(o: Point3<S>, d: Vec3<S>, t_max: f32) -> Self {
        Self { o, d, t_max }
    }

    pub fn point_at(&self, t: f32) -> Point3<S> {
        self.o + self.d * t
    }
}
