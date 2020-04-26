use crate::spectrum::Wavelength;

// Balance heuristic
pub fn hwss_weight(hero_wavelength: Wavelength, pdf_xi_given_lambdas: [f32; 4]) -> f32 {
    let pdf_lambdas = [
        hero_wavelength.rotate_n(0).pdf(),
        hero_wavelength.rotate_n(1).pdf(),
        hero_wavelength.rotate_n(2).pdf(),
        hero_wavelength.rotate_n(3).pdf(),
    ];

    let numerator = pdf_lambdas[0] * pdf_xi_given_lambdas[0];

    // Pls change this :(
    let denominator: f32 = pdf_lambdas
        .iter()
        .zip(pdf_xi_given_lambdas.iter())
        .map(|(&pdf_lambda, &pdf_xi_given_lambda)| pdf_lambda * pdf_xi_given_lambda)
        .sum();

    numerator / denominator
}

pub fn balance_heuristic(f: [f32; 4], g: [f32; 4]) -> f32 {
    let f_sum: f32 = f.iter().sum();
    let g_sum: f32 = g.iter().sum();

    f[0] / (f_sum + g_sum)
}
