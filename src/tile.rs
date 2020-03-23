use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;

use crate::math::{Point3, Ray, Vec3};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

const MAX_TILE_WIDTH: usize = 64;
const MAX_TILE_HEIGHT: usize = 64;
const TOTAL_SPP: usize = 500;
const SAMPLE_CHUNK_SIZE: usize = 20;

fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn get_pixel_color(x: usize, y: usize, spp: usize) -> (f32, f32, f32) {
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);

    let dir = Vec3::new(0.0, 0.0, -1.0);
    let pixel_center = Point3::new(
        ((x as f32 + 0.5) / (WIDTH as f32) - 0.5) * 2.0,
        ((y as f32 + 0.5) / (HEIGHT as f32) - 0.5) * -2.0,
        0.0,
    );
    let w = 1.0 / TOTAL_SPP as f32;

    for _ in 0..spp {
        let jitter = Vec3::new(
            rand::random::<f32>() / WIDTH as f32,
            rand::random::<f32>() / HEIGHT as f32,
            0.0,
        );
        let (ar, ag, ab) = trace_ray(Ray::new(pixel_center + jitter, dir));

        r += ar.max(0.0) * w;
        g += ag.max(0.0) * w;
        b += ab.max(0.0) * w;
    }

    (r, g, b)
}

fn trace_ray(ray: Ray) -> (f32, f32, f32) {
    if ray.o.x * ray.o.x + ray.o.y * ray.o.y <= 0.6 {
        (0.7, 0.2, 0.2)
    } else {
        (0.1, 0.1, 0.1)
    }
}


#[derive(Debug, Clone)]
pub struct TileData {
    pub idx: usize,
    pub remaining_samples: usize,
    pub framebuffer: Vec<(f32, f32, f32)>,
}

impl TileData {
    pub fn render(mut self, render_buffer: &RwLock<Vec<u32>>) -> Self {
        let tile_width = (WIDTH / 4).min(MAX_TILE_WIDTH);
        let tile_height = (HEIGHT / 4).min(MAX_TILE_HEIGHT);

        let num_horiz_tiles = (WIDTH + tile_width - 1) / tile_width;
        let num_vert_tiles = (HEIGHT + tile_height - 1) / tile_height;
        let num_tiles = num_vert_tiles * num_horiz_tiles;

        let tile_x = self.idx % num_horiz_tiles;
        let tile_y = self.idx / num_horiz_tiles;

        let this_tile_width = WIDTH - ((num_horiz_tiles.max(1) - 1) * tile_width);
        let this_tile_height = HEIGHT - ((num_vert_tiles.max(1) - 1) * tile_height);
        let this_tile_pixels = this_tile_height * this_tile_width;

        let pixel_start_x = tile_x * tile_width;
        let pixel_start_y = tile_y * tile_height;

        let mut temp_buffer = vec![0; this_tile_pixels];
        let start_time = Instant::now();

        while self.remaining_samples > 0 && Instant::now().saturating_duration_since(start_time).as_secs_f32() <= 0.25 {
            let new_remaining_samples = self.remaining_samples.saturating_sub(SAMPLE_CHUNK_SIZE);
            let samples_this_iter = self.remaining_samples - new_remaining_samples;

            for (i, p) in temp_buffer.iter_mut().enumerate() {
                let (nr, ng, nb) = get_pixel_color(
                    pixel_start_x + i % this_tile_width,
                    pixel_start_y + i / this_tile_width,
                    samples_this_iter,
                );

                let (or, og, ob) = self.framebuffer[i];
                let (ar, ag, ab) = (or + nr, og + ng, ob + nb);
                self.framebuffer[i] = (ar, ab, ag);

                let w2 = (TOTAL_SPP as f32) / ((TOTAL_SPP - new_remaining_samples) as f32) * 255.99;
                *p = rgb_to_u32((ar * w2).min(255.99) as u8, (ag * w2).min(255.99) as u8, (ab * w2).min(255.99) as u8);
            }

            self.remaining_samples = new_remaining_samples;
        }

        let mut render_buffer = render_buffer.write().unwrap();
        for i in 0..this_tile_height {
            let abs = (pixel_start_y + i) * WIDTH + pixel_start_x;
            render_buffer[abs..(abs + this_tile_width)]
                .copy_from_slice(&temp_buffer[(i * this_tile_width)..((i + 1) * this_tile_width)]);
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

