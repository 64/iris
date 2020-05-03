use crate::{
    math::{Shading, Vec3},
    sampling::Sampler,
};

// http://jcgt.org/published/0007/04/01/paper.pdf#page=10
pub fn sample(
    wo: Vec3<Shading>,
    alpha_x: f32,
    alpha_y: f32,
    sampler: &mut Sampler,
) -> Vec3<Shading> {
    let wo = Vec3::<Shading>::new(alpha_x * wo.x(), alpha_y * wo.y(), wo.z()).normalize();

    let len_2 = wo.x().powi(2) + wo.y().powi(2);
    let t1_v = if len_2 > 0.0 {
        Vec3::new(-wo.y(), wo.x(), 0.0) / len_2.sqrt()
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let t2_v = wo.cross(t1_v);

    let r = sampler.gen_0_1().sqrt();
    let phi = 2.0 * std::f32::consts::PI * sampler.gen_0_1();
    let t1 = r * phi.cos();
    let t2 = r * phi.sin();
    let s = 0.5 * (1.0 + wo.z());
    let t2 = (1.0 - s) * (1.0 - t1.powi(2)).sqrt() + s * t2;

    let nh = t1 * t1_v + t2 * t2_v + wo * (1.0 - t1.powi(2) - t2.powi(2)).max(0.0).sqrt();

    Vec3::new(alpha_x * nh.x(), alpha_y * nh.y(), nh.z().max(0.0)).normalize()
}

pub fn evaluate(wh: Vec3<Shading>, alpha_x: f32, alpha_y: f32) -> f32 {
    if wh.tan_2_theta().is_infinite() {
        return 0.0;
    }

    let (ax, ay) = (alpha_x, alpha_y);
    let e = (wh.cos_2_phi() / ax.powi(2) + wh.sin_2_phi() / ay.powi(2)) * wh.tan_2_theta();
    1.0 / (std::f32::consts::PI * ax * ay * wh.cos_theta().powi(4) * (1.0 + e).powi(2))
}

fn lambda(w: Vec3<Shading>, alpha_x: f32, alpha_y: f32) -> f32 {
    if w.tan_theta().is_infinite() {
        return 0.0;
    }

    let (ax, ay) = (alpha_x, alpha_y);
    let alpha = (w.cos_2_phi() * ax.powi(2) + w.sin_2_phi() * ay.powi(2)).sqrt();
    let alpha_2_tan_2_theta = (alpha * w.tan_theta().abs()).powi(2); // Do we need to abs?

    (-1.0 + (1.0 + alpha_2_tan_2_theta).sqrt()) / 2.0
}

pub fn g(wo: Vec3<Shading>, wh: Vec3<Shading>, alpha_x: f32, alpha_y: f32) -> f32 {
    1.0 / (1.0 + lambda(wh, alpha_x, alpha_y) + lambda(wo, alpha_x, alpha_y))
}

pub fn pdf(wo: Vec3<Shading>, wh: Vec3<Shading>, alpha_x: f32, alpha_y: f32) -> f32 {
    evaluate(wh, alpha_x, alpha_y) * g1(wo, alpha_x, alpha_y) * wo.dot(wh).abs()
        / wo.cos_theta().abs()
}

pub fn roughness_to_alpha(r: f32) -> f32 {
    let x = r.max(1e-3).ln();
    1.62142
        + 0.819_955 * x
        + 0.1734 * x.powi(2)
        + 0.017_120_1 * x.powi(3)
        + 0.000_640_711 * x.powi(4)
}

fn g1(w: Vec3<Shading>, alpha_x: f32, alpha_y: f32) -> f32 {
    1.0 / (1.0 + lambda(w, alpha_x, alpha_y))
}
