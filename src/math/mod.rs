mod matrix;
mod point3;
mod ray;
mod vec3;

pub use matrix::*;
pub use point3::*;
pub use ray::*;
pub use vec3::*;

pub mod mis;

#[derive(Debug, Copy, Clone)]
pub struct Global;
