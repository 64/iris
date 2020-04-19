mod matrix;
mod ord_float;
mod point3;
mod ray;
mod vec3;

pub use matrix::*;
pub use ord_float::*;
pub use point3::*;
pub use ray::*;
pub use vec3::*;

// Coordinate spaces

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct World;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Clip;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Shading;
