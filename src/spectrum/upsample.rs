// Implementation of http://rgl.epfl.ch/publications/Jakob2019Spectral
use crate::spectrum::SampleableSpectrum;
use std::{convert::TryInto, fs::File, io::Read};

pub struct UpsampleTable {
    coefficients: Vec<f32>,
    scale: Vec<f32>,
    resolution: usize,
}

#[derive(Debug, Clone)]
pub struct UpsampledSpectrum {
    coefficients: [f32; 3],
}

impl SampleableSpectrum for UpsampledSpectrum {
    // TODO: Simd
    fn evaluate_single(&self, lambda: f32) -> f32 {
        let x = self.coefficients[0]
            .mul_add(lambda, self.coefficients[1])
            .mul_add(lambda, self.coefficients[2]);
        let y = 1.0 / x.mul_add(x, 1.0).sqrt();
        (0.5 * x).mul_add(y, 0.5)
    }
}

#[derive(Debug, Clone)]
pub struct UpsampledHdrSpectrum {
    coefficients: [f32; 3],
    hdr_coefficient: f32,
}

impl SampleableSpectrum for UpsampledHdrSpectrum {
    // TODO: Simd
    fn evaluate_single(&self, lambda: f32) -> f32 {
        let x = self.coefficients[0]
            .mul_add(lambda, self.coefficients[1])
            .mul_add(lambda, self.coefficients[2]);
        let y = 1.0 / x.mul_add(x, 1.0).sqrt();
        self.hdr_coefficient * (0.5 * x).mul_add(y, 0.5)
    }
}

impl UpsampleTable {
    #[allow(unused)]
    pub fn get_spectrum_hdr(&self, rgb: [f32; 3]) -> UpsampledHdrSpectrum {
        let max = rgb[0].max(rgb[1]).max(rgb[2]).max(1.0);

        UpsampledHdrSpectrum {
            coefficients: self
                .get_spectrum([rgb[0] / max, rgb[1] / max, rgb[2] / max])
                .coefficients,
            hdr_coefficient: max.min(50.0), /* Remove this hack once we importance sample the
                                             * env map */
        }
    }

    // TODO: Make not C-ish
    pub fn get_spectrum(&self, rgb: [f32; 3]) -> UpsampledSpectrum {
        let mut i = 0;
        let res = self.resolution;
        for j in 1..3 {
            if rgb[j] >= rgb[i] {
                i = j;
            }
        }

        let z = rgb[i];
        let scale = (res - 1) as f32 / z;
        let x = rgb[(i + 1) % 3] * scale;
        let y = rgb[(i + 2) % 3] * scale;

        let xi = (x as usize).min(res - 2);
        let yi = (y as usize).min(res - 2);
        let zi = find_interval(&self.scale, self.resolution, z);
        let mut offset = (((i * res + zi) * res + yi) * res + xi) * 3;
        let dx = 3;
        let dy = 3 * res;
        let dz = 3 * res.pow(2);

        let x1 = x - xi as f32;
        let x0 = 1.0 - x1;
        let y1 = y - yi as f32;
        let y0 = 1.0 - y1;
        let z1 = (z - self.scale[zi]) / (self.scale[zi + 1] - self.scale[zi]);
        let z0 = 1.0 - z1;

        let mut coefficients = [0.0; 3];

        for out in &mut coefficients {
            *out = ((self.coefficients[offset] * x0 + self.coefficients[offset + dx] * x1) * y0
                + (self.coefficients[offset + dy] * x0 + self.coefficients[offset + dy + dx] * x1)
                    * y1)
                * z0
                + ((self.coefficients[offset + dz] * x0
                    + self.coefficients[offset + dz + dx] * x1)
                    * y0
                    + (self.coefficients[offset + dz + dy] * x0
                        + self.coefficients[offset + dz + dy + dx] * x1)
                        * y1)
                    * z1;
            offset += 1;
        }

        UpsampledSpectrum { coefficients }
    }

    pub fn load() -> Self {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/srgb.coeff");

        let mut file = File::open(path).expect("failed to load srgb coefficients");

        let mut signature = [0u8; 4];
        file.read_exact(&mut signature).unwrap();
        if &signature != b"SPEC" {
            panic!("sRGB upscaling coefficients file: incorrect header");
        }

        let mut resolution = [0u8; 4];
        file.read_exact(&mut resolution).unwrap();
        let resolution = u32::from_le_bytes(resolution) as usize;

        let mut scale = vec![0; resolution * std::mem::size_of::<f32>()];
        file.read_exact(&mut scale).unwrap();

        let scale = scale
            .chunks_exact(4)
            .map(|slice| f32::from_le_bytes(slice.try_into().unwrap()))
            .collect::<Vec<f32>>();

        let mut coefficients = Vec::new();
        file.read_to_end(&mut coefficients).unwrap();

        let coefficients = coefficients
            .chunks_exact(4)
            .map(|slice| f32::from_le_bytes(slice[0..4].try_into().unwrap()))
            .collect::<Vec<f32>>();

        Self {
            coefficients,
            scale,
            resolution,
        }
    }
}

fn find_interval(values: &[f32], size: usize, x: f32) -> usize {
    let mut left = 0;
    let last_interval = size - 2;
    let mut size = last_interval;

    while size > 0 {
        let half = size / 2;
        let middle = left + half + 1;

        if values[middle] < x {
            left = middle;
            size -= half + 1;
        } else {
            size = half;
        }
    }

    left.min(last_interval)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsample() {
        let table = UpsampleTable::load();

        let spectrum = table.get_spectrum([0.0, 1.0, 0.0]);
        assert_eq!(
            0.010374308,
            spectrum.evaluate_single(Wavelength::new(450.0))
        );
        assert_eq!(
            0.021721054,
            spectrum.evaluate_single(Wavelength::new(460.0))
        );
        assert_eq!(0.95374036, spectrum.evaluate_single(Wavelength::new(520.0)));
        assert_eq!(0.16656497, spectrum.evaluate_single(Wavelength::new(600.0)));
    }
}
