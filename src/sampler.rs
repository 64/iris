// Only implements the Sobol sampler, for now
// Code adapted from psychopath renderer (see crates/sobol for LICENSE.md)
#[derive(Debug, Clone)]
pub struct Sampler {
    scramble: u32,
    dimension: u32,
    index: u32,
}

impl Sampler {
    pub fn new(x: usize, y: usize, sample_index: usize, seed: u32) -> Self {
        Self {
            scramble: hash_u32((x as u32) ^ ((y as u32) << 16), seed),
            dimension: 0,
            index: sample_index as u32,
        }
    }

    pub fn gen_0_1(&mut self) -> f32 {
        let sample = if self.dimension < sobol::MAX_DIMENSION {
            sobol::sample_owen(
                self.dimension,
                self.index,
                hash_u32(self.dimension, self.scramble),
            )
        } else {
            hash_u32_to_f32(self.dimension ^ (self.index << 16), self.scramble)
        };

        self.dimension += 1;

        sample
    }

    pub fn gen_range(&mut self, lower: f32, upper: f32) -> f32 {
        debug_assert!(lower < upper);
        self.gen_0_1() * (upper - lower) + lower
    }

    pub fn sample_with_pdf<S: Sampleable + Clone>(&mut self) -> (S, f32) {
        S::sample_with_pdf(self)
    }
}

pub trait Sampleable: Sized {
    fn sample(sampler: &mut Sampler) -> Self;
    fn pdf(val: &Self) -> f32;

    fn sample_with_pdf(sampler: &mut Sampler) -> (Self, f32) {
        let sample = Self::sample(sampler);
        let pdf = Self::pdf(&sample);
        (sample, pdf)
    }
}

fn hash_u32(n: u32, seed: u32) -> u32 {
    let mut hash = n;
    for _ in 0..3 {
        hash = hash.wrapping_mul(1_936_502_639);
        hash ^= hash.wrapping_shr(16);
        hash ^= seed;
    }

    hash
}

fn hash_u32_to_f32(n: u32, seed: u32) -> f32 {
    const INV_MAX: f32 = 1.0 / std::u32::MAX as f32;
    hash_u32(n, seed) as f32 * INV_MAX
}
