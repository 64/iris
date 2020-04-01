pub fn hwss_weight(
    pdf_lambda_h: f32,
    pdf_xi_given_lambda_h: f32,
    pdf_lambdas: [f32; 4],
    pdf_xi_given_lambdas: [f32; 4],
) -> f32 {
    // We don't square the numerator here, but compensate by not dividing the sample
    // f(X, lambda) by p(X, lambda) in tile.rs.
    let numerator = pdf_lambda_h * pdf_xi_given_lambda_h;
    let denominator: f32 = pdf_lambdas
        .iter()
        .zip(pdf_xi_given_lambdas.iter())
        .map(|(&pdf_lambda, &pdf_xi_given_lambda)| pdf_lambda * pdf_xi_given_lambda)
        .map(|x| x.powi(2))
        .sum();

    numerator / denominator
}
