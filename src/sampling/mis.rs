use crate::math::PdfSet;

pub fn balance_heuristic_1(f: PdfSet) -> f32 {
    f.hero() / f.sum()
}

// TODO: Power heuristic
pub fn balance_heuristic_2(f: PdfSet, g: PdfSet) -> f32 {
    f.hero() / (f + g).sum()
}
