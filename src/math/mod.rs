mod matrix;
mod pdf;
mod point3;
mod ray;
mod vec3;

pub use matrix::*;
pub use pdf::*;
pub use point3::*;
pub use ray::*;
pub use vec3::*;

pub use ray::offset_origin;

// Coordinate spaces

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct World;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Local;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Clip;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Shading;

// TODO: Find a better place to put this
pub fn fresnel_dielectric(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    let cos_theta_i = f32::clamp(cos_theta_i, -1.0, 1.0);
    assert!(cos_theta_i > 0.0, "cos_theta_i: {}", cos_theta_i);

    let sin_theta_i = (1.0 - cos_theta_i.powi(2)).max(0.0).sqrt();
    let sin_theta_t = eta_i / eta_t * sin_theta_i;

    // TIR
    if sin_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin_theta_t.powi(2)).max(0.0).sqrt();
    let r_par = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t))
        / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
    let r_perp = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t))
        / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));

    (r_par.powi(2) + r_perp.powi(2)) / 2.0
}
