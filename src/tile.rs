use std::{
    cmp::{Ord, Ordering},
};

use crate::{
    color::Xyz,
    math::{Point3, Ray, Vec3},
    sampling::Sampler,
    spectrum::Wavelength,
    Render,
};

const MAX_TILE_WIDTH: usize = 64;
const MAX_TILE_HEIGHT: usize = 64;
//const SAMPLE_CHUNK_SIZE: usize = 5000;
const SEED: u32 = 123_456_789;

#[derive(Debug, Clone)]
pub struct TileData {
    pub idx: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_x: usize,
    pub pixel_y: usize,
    pub distance_from_center: f32,
    pub remaining_samples: usize,
    pub accum_buffer: Vec<Xyz>,
    pub temp_buffer: Vec<(f32, f32, f32)>,
}

// TODO: This code is very messy and I am not particularly happy with it

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

        let pixel_center_x = (pixel_start_x + this_tile_width / 2) as i32;
        let pixel_center_y = (pixel_start_y + this_tile_height / 2) as i32;
        let distance_from_center_x = (pixel_center_x - render.width as i32 / 2) as f32;
        let distance_from_center_y = (pixel_center_y - render.height as i32 / 2) as f32;
        let distance_from_center =
            (distance_from_center_x.powi(2) + distance_from_center_y.powi(2)).sqrt();

        Some(TileData {
            idx,
            distance_from_center,
            width: this_tile_width,
            height: this_tile_height,
            pixel_x: pixel_start_x,
            pixel_y: pixel_start_y,
            remaining_samples: render.spp,
            accum_buffer: vec![Xyz::new(0.0, 0.0, 0.0); this_tile_pixels],
            temp_buffer: vec![(0.0, 0.0, 0.0); this_tile_pixels],
        })
    }

    pub fn render(mut self, render: &Render) -> Self {
        //use time::Instant;
        //let start_time = Instant::now();

        // The below is needed for progressive rendering
        while self.remaining_samples > 0
            //&& Instant::now()
                //.saturating_duration_since(start_time)
                //.as_secs_f32()
                //<= 0.1
        {
            //let new_remaining_samples = self.remaining_samples.saturating_sub(SAMPLE_CHUNK_SIZE);
            let new_remaining_samples = 0;
            let samples_this_iter = self.remaining_samples - new_remaining_samples;
            let weight = render.spp as f32 / ((render.spp - new_remaining_samples) as f32);

            for (i, (accumulator, pixel)) in self
                .accum_buffer
                .iter_mut()
                .zip(self.temp_buffer.iter_mut())
                .enumerate()
            {
                let xyz = get_pixel_color(
                    self.pixel_x + i % self.width,
                    self.pixel_y + i / self.width,
                    samples_this_iter,
                    render.spp - self.remaining_samples,
                    render,
                );

                *accumulator += xyz;

                let rgb = (*accumulator * weight).to_rgb_hdr();
                *pixel = (rgb.0.max(0.0), rgb.1.max(0.0), rgb.2.max(0.0));
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

fn get_pixel_color(
    x_abs: usize,
    y_abs: usize,
    samples_this_iter: usize,
    samples_so_far: usize,
    render: &Render,
) -> Xyz {
    let pixel_center_clip = Point3::new(
        ((x_abs as f32 + 0.5) / (render.width as f32) - 0.5) * 2.0,
        ((y_abs as f32 + 0.5) / (render.height as f32) - 0.5) * -2.0,
        0.0,
    );

    let weight = 1.0 / render.spp as f32;

    let mut xyz_sum = Xyz::new(0.0, 0.0, 0.0);

    for i in 0..samples_this_iter {
        let mut sampler = Sampler::new(x_abs, y_abs, i + samples_so_far, SEED);

        let hero_wavelength = Wavelength::sample(&mut sampler);

        let jitter_clip = Vec3::new(
            0.5 * sampler.gen_0_1() / render.width as f32,
            0.5 * sampler.gen_0_1() / render.height as f32,
            0.0,
        );

        let target_world = &render.camera.clip_to_world * (pixel_center_clip + jitter_clip);
        let origin_world = render.camera.position;
        let ray = Ray::new(origin_world, target_world - origin_world);

        xyz_sum += render
            .scene
            .radiance(ray, hero_wavelength, &mut sampler)
            .to_xyz(hero_wavelength);
    }

    xyz_sum * weight
}

impl PartialEq for TileData {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl Eq for TileData {}

impl PartialOrd for TileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TileData {
    fn cmp(&self, other: &Self) -> Ordering {
        // Render from the inside out
        self.remaining_samples.cmp(&other.remaining_samples).then(
            self.distance_from_center
                .partial_cmp(&other.distance_from_center)
                .unwrap()
                .reverse(),
        )
    }
}
