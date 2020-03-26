use rand::{thread_rng, Rng};

use std::{
    cmp::{Ord, Ordering},
    time::Instant,
};

use crate::{
    math::{Point3, Ray, Vec3},
    render::Render,
    spectrum::{self, SpectrumSample},
    color::Xyz,
};

const MAX_TILE_WIDTH: usize = 64;
const MAX_TILE_HEIGHT: usize = 64;
const SAMPLE_CHUNK_SIZE: usize = 20;

// TODO: Move to color.rs
fn get_pixel_color(
    x: usize,
    y: usize,
    spp_this_iter: usize,
    render: &Render,
) -> Xyz {
    let dir = Vec3::new(0.0, 0.0, -1.0);
    let pixel_center = Point3::new(
        ((x as f32 + 0.5) / (render.width as f32) - 0.5) * 2.0,
        ((y as f32 + 0.5) / (render.height as f32) - 0.5) * -2.0,
        0.0,
    );
    let weight = 1.0 / render.spp as f32;

    let mut xyz_sum = Xyz::new(0.0, 0.0, 0.0);

    for _ in 0..spp_this_iter {
        let jitter = Vec3::new(
            rand::random::<f32>() / render.width as f32,
            rand::random::<f32>() / render.height as f32,
            0.0,
        );

        xyz_sum += trace_ray(Ray::new(pixel_center + jitter, dir)).to_xyz();
    }

    xyz_sum * weight
}

fn trace_ray(ray: Ray) -> SpectrumSample {
    if ray.o.x * ray.o.x + ray.o.y * ray.o.y <= 0.6 {
        let mean = 700.0;
        let var = 30.0;

        use statrs::distribution::{Continuous, Normal};
        let wavelength = thread_rng().gen_range(spectrum::LAMBDA_MIN_NM, spectrum::LAMBDA_MAX_NM);
        let dist = Normal::new(mean, var).unwrap();
        let value = 50.0 * dist.pdf(wavelength as f64) / dist.pdf(mean);
        SpectrumSample::new(wavelength as f32, value as f32)
    } else {
        SpectrumSample::new(
            thread_rng().gen_range(spectrum::LAMBDA_MIN_NM, spectrum::LAMBDA_MAX_NM),
            0.5,
        )
    }
}

#[derive(Debug, Clone)]
pub struct TileData {
    pub idx: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_x: usize,
    pub pixel_y: usize,
    pub remaining_samples: usize,
    pub accum_buffer: Vec<Xyz>,
    pub temp_buffer: Vec<u32>,
}

impl TileData {
    pub fn new(render: &Render, idx: usize) -> Option<Self> {
        let tile_width = (render.width / 4).min(MAX_TILE_WIDTH);
        let tile_height = (render.height / 4).min(MAX_TILE_HEIGHT);

        let num_horiz_tiles = (render.width + tile_width - 1) / tile_width;
        let num_vert_tiles = (render.height + tile_height - 1) / tile_height;
        let num_tiles = num_vert_tiles * num_horiz_tiles;

        if idx >= num_tiles {
            return None;
        }

        let this_tile_width = render.width - ((num_horiz_tiles.max(1) - 1) * tile_width);
        let this_tile_height = render.height - ((num_vert_tiles.max(1) - 1) * tile_height);
        let this_tile_pixels = this_tile_height * this_tile_width;

        let tile_x = idx % num_horiz_tiles;
        let tile_y = idx / num_horiz_tiles;

        let pixel_start_x = tile_x * tile_width;
        let pixel_start_y = tile_y * tile_height;

        Some(TileData {
            idx,
            width: this_tile_width,
            height: this_tile_height,
            pixel_x: pixel_start_x,
            pixel_y: pixel_start_y,
            remaining_samples: render.spp,
            accum_buffer: vec![Xyz::new(0.0, 0.0, 0.0); this_tile_pixels],
            temp_buffer: vec![0; this_tile_pixels],
        })
    }

    pub fn render(mut self, render: &Render) -> Self {
        let start_time = Instant::now();

        while self.remaining_samples > 0
            && Instant::now()
                .saturating_duration_since(start_time)
                .as_secs_f32()
                <= 0.1
        {
            let new_remaining_samples = self.remaining_samples.saturating_sub(SAMPLE_CHUNK_SIZE);
            let samples_this_iter = self.remaining_samples - new_remaining_samples;

            let weight = ((spectrum::LAMBDA_MAX_NM - spectrum::LAMBDA_MIN_NM))
                / ((render.spp - new_remaining_samples) as f32);

            for (i, p) in self.temp_buffer.iter_mut().enumerate() {
                let xyz = get_pixel_color(
                    self.pixel_x + i % self.width,
                    self.pixel_y + i / self.width,
                    samples_this_iter,
                    render,
                );

                self.accum_buffer[i] += xyz;

                *p = (self.accum_buffer[i] * weight).to_srgb().to_u32();
            }

            self.remaining_samples = new_remaining_samples;
        }

        let mut render_buffer = render.buffer.write().unwrap();

        for i in 0..self.height {
            let abs = (self.pixel_y + i) * render.width + self.pixel_x;
            render_buffer[abs..(abs + self.width)]
                .copy_from_slice(&self.temp_buffer[(i * self.width)..((i + 1) * self.width)]);
        }

        self
    }
}

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Eq for TileData {}

impl PartialOrd for TileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.remaining_samples.partial_cmp(&other.remaining_samples)
    }
}

impl Ord for TileData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.remaining_samples.cmp(&other.remaining_samples)
    }
}
