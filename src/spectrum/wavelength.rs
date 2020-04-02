use crate::sampler::Sampler;

pub const LAMBDA_MIN_NM: f32 = 380.0;
pub const LAMBDA_MAX_NM: f32 = 700.0;
pub const LAMBDA_RANGE_NM: f32 = LAMBDA_MAX_NM - LAMBDA_MIN_NM;

#[derive(Debug, Copy, Clone)]
pub struct Wavelength(f32);

impl Wavelength {
    pub fn as_nm_f32(self) -> f32 {
        self.0
    }

    pub fn rotate_n(self, n: usize) -> Wavelength {
        let lambda = self.0 + (LAMBDA_RANGE_NM / 4.0) * (n as f32);

        // Perform modulo operation (so that lambda is always in range)
        if lambda >= LAMBDA_MAX_NM {
            Self(lambda - LAMBDA_RANGE_NM)
        } else {
            Self(lambda)
        }
    }

    // Importance sample the wavelength with pdf(lambda) = sech^2(0.0072(lambda -
    // 538)) See https://www.researchgate.net/publication/228938842_An_Improved_Technique_for_Full_Spectral_Rendering
    // Even though we stratify the spectrum for HWSS, it's important that the hero
    // wavelength is importance sampled too because we use it for path
    // generation.
    pub fn sample(sampler: &mut Sampler) -> Self {
        Wavelength(inverse_cdf(sampler.gen_0_1()))
    }

    pub fn pdf(self) -> f32 {
        pdf(self.0)
    }
}

fn pdf(lambda: f32) -> f32 {
    if lambda < 380.0 || lambda > 700.0 {
        return 0.0;
    }
    // Normalization constant comes from:
    // https://www.wolframalpha.com/input/?i=integral+sech%5E2%280.0072%28y-538%29%29+between+380+and+700
    (0.0072 * (lambda - 538.0)).cosh().powi(-2) / 227.322
}

fn cdf_partial(lambda: f32) -> f32 {
    // https://www.wolframalpha.com/input/?i=indefinite+integral+sech%5E2%280.0072%28y-538%29%29%2F227.322
    0.0253855 * (0.0072 * lambda).sinh() / (3.8736 - 0.0072 * lambda).cosh()
}

fn cdf(lambda: f32) -> f32 {
    (cdf_partial(lambda) - cdf_partial(LAMBDA_MIN_NM)).clamp(0.0, 1.0)
}

fn inverse_cdf(unif: f32) -> f32 {
    // Newton's method
    // Is this guess good?
    let mut x = 500.0;

    // Allow up to 0.5% error (is this reasonable?)
    while (cdf(x) - unif).abs() > 0.005 {
        x = x - (cdf(x) - unif) / pdf(x);
    }

    x
}
