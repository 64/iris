#![allow(dead_code)]
#![allow(unused)]
use crate::{
    bsdf::SampleableBsdf,
    math::{Shading, Vec3},
    sampling::{self, ggx, Sampler},
    spectrum::{SampleableSpectrum, SpectralSample, Spectrum, Wavelength},
};

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct MicrofacetBsdf {
    reflectance: Spectrum,
    alpha_x: f32,
    alpha_y: f32,
}

impl MicrofacetBsdf {
    pub fn new<S: Into<Spectrum>>(reflectance: S, roughness_x: f32, roughness_y: f32) -> Self {
        Self {
            reflectance: reflectance.into(),
            alpha_x: ggx::roughness_to_alpha(roughness_x),
            alpha_y: ggx::roughness_to_alpha(roughness_y),
        }
    }
}

impl SampleableBsdf for MicrofacetBsdf {
    fn evaluate(
        &self,
        wi: Vec3<Shading>,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
    ) -> SpectralSample {
        let cos_theta_o = wo.cos_theta().abs();
        let cos_theta_i = wi.cos_theta().abs();
        let wh = wo + wi;

        if !wi.same_hemisphere(wo)
            || cos_theta_o == 0.0
            || cos_theta_i == 0.0
            || wh == Vec3::splat(0.0)
        {
            return SpectralSample::splat(0.0);
        }

        assert!(wi.z() > 0.0, "{:?}", wi);

        let wh = wh.normalize();
        let wh_facing = wh.face_forward(Vec3::new(0.0, 0.0, 1.0));
        let d = ggx::evaluate(wh, self.alpha_x, self.alpha_y);
        let f = fresnel_dielectric(wi.dot(wh_facing), 1.5, 1.0);
        let g = ggx::g(wo, wh, self.alpha_x, self.alpha_y);
        self.reflectance.evaluate(hero_wavelength) * d * f * g / (4.0 * cos_theta_o * cos_theta_i)
    }

    fn pdf(&self, wi: Vec3<Shading>, wo: Vec3<Shading>, hero_wavelength: Wavelength) -> [f32; 4] {
        let wh = (wi + wo).normalize();
        let res = ggx::pdf(wo, wh, self.alpha_x, self.alpha_y) / (4.0 * wo.dot(wh));
        [res; 4]
    }

    fn sample(
        &self,
        wo: Vec3<Shading>,
        hero_wavelength: Wavelength,
        sampler: &mut Sampler,
    ) -> (Vec3<Shading>, [f32; 4]) {
        let wh = ggx::sample(wo, self.alpha_x, self.alpha_y, sampler);
        let wi = reflect(wo, wh);

        if wo.cos_theta() == 0.0 || wo.dot(wh) < 0.0 || !wo.same_hemisphere(wi) {
            return (Vec3::splat(0.0), [0.0; 4]);
        }

        assert!(wi.z() > 0.0, "{:?}, {:?}", wi, wh);

        let pdf = ggx::pdf(wo, wh, self.alpha_x, self.alpha_y) / (4.0 * wo.dot(wh));
        (wi, [pdf; 4])
    }
}

fn reflect(wo: Vec3<Shading>, n: Vec3<Shading>) -> Vec3<Shading> {
    -wo + (2.0 * wo.dot(n) * n)
}

fn fresnel_dielectric(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
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
