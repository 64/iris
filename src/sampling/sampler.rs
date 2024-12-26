// Only implements the Sobol sampler, for now
// Code adapted from psychopath renderer (see crates/sobol for LICENSE.md)
#[derive(Debug, Clone)]
pub struct Sampler {
    scramble: u32,
    dimension: u32,
    index: u32,
    sample_buffer: [f32; 4],
    samples_in_buffer: usize,
}

impl Sampler {
    pub fn new(x: usize, y: usize, sample_index: usize, seed: u32) -> Self {
        Self {
            scramble: hash_u32((x as u32) ^ ((y as u32) << 16), seed),
            dimension: 0,
            index: sample_index as u32,
            sample_buffer: [0.0; 4],
            samples_in_buffer: 0,
        }
    }

    pub fn gen_0_1(&mut self) -> f32 {
        if self.samples_in_buffer > 0 {
            self.samples_in_buffer -= 1;
            self.sample_buffer[self.samples_in_buffer]
        } else if self.samples_in_buffer == 0 {
            // Buffer empty, refill
            self.sample_buffer = if self.dimension < sobol::NUM_DIMENSIONS {
                sobol::sample_4d(
                    self.index,
                    self.dimension,
                    hash_u32(self.dimension, self.scramble),
                )
            } else {
                [
                    hash_u32_to_f32((self.dimension + 0) ^ (self.index << 16), self.scramble),
                    hash_u32_to_f32((self.dimension + 1) ^ (self.index << 16), self.scramble),
                    hash_u32_to_f32((self.dimension + 2) ^ (self.index << 16), self.scramble),
                    hash_u32_to_f32((self.dimension + 3) ^ (self.index << 16), self.scramble),
                ]
            };

            self.dimension += 1;
            self.samples_in_buffer = 3;
            self.sample_buffer[3]
        } else {
            unreachable!()
        }
    }

    pub fn gen_range(&mut self, lower: f32, upper: f32) -> f32 {
        debug_assert!(lower < upper);
        self.gen_0_1() * (upper - lower) + lower
    }

    pub fn gen_array_index(&mut self, len: usize) -> usize {
        debug_assert!(len > 0);
        self.gen_range(0.0, len as f32 - 0.5) as usize
    }

    // https://www.graphics.rwth-aachen.de/publication/2/jgt.pdf
    // https://github.com/cessen/psychopath/blob/0dfe916523c00a0979de6fb1f781c3a51ba81113/src/renderer.rs#L728
    // Only works once per sampler
    pub fn gen_golden_ratio(&self) -> f32 {
        let uniform_integer = self
            .index
            .wrapping_add(self.scramble)
            .wrapping_mul(2654435769);
        (uniform_integer as f32) / (u32::MAX as f32)
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
