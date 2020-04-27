#![allow(dead_code)]

pub mod ggx;
pub mod mis;

mod sampler;
pub use sampler::Sampler;

use crate::math::Vec3;
use std::f32::consts::PI;

pub fn unit_hemisphere<S>(r1: f32, r2: f32) -> Vec3<S> {
    let (sin, cos) = (2.0 * PI * r1).sin_cos();
    let hyp = (1.0 - r2.powi(2)).sqrt();
    Vec3::new(cos * hyp, sin * hyp, r2)
}

pub fn pdf_unit_hemisphere() -> f32 {
    1.0 / (2.0 * PI)
}

pub fn pdf_cone(cos_theta_max: f32) -> f32 {
    1.0 / (2.0 * PI * (1.0 - cos_theta_max))
}

fn concentric_disk(r1: f32, r2: f32) -> (f32, f32) {
    let x_off = 2.0 * r1 - 1.0;
    let y_off = 2.0 * r2 - 1.0;

    if x_off == 0.0 && y_off == 0.0 {
        return (0.0, 0.0);
    }

    let (r, theta) = if x_off.abs() > y_off.abs() {
        (x_off, PI / 4.0 * (y_off / x_off))
    } else {
        (y_off, (PI / 2.0) - (PI / 4.0) * (x_off / y_off))
    };

    (r * theta.cos(), r * theta.sin())
}

pub fn cosine_unit_hemisphere<S>(r1: f32, r2: f32) -> Vec3<S> {
    let (x, y) = concentric_disk(r1, r2);
    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
    Vec3::new(x, y, z)
}

pub fn pdf_cosine_unit_hemisphere(cos_theta: f32) -> f32 {
    debug_assert!(cos_theta >= 0.0);
    cos_theta / PI
}
