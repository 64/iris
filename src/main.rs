use minifb::{Key, Window, WindowOptions};

use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;

mod frame;
mod math;

use math::{Point3, Ray, Vec3};

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
    //dbg!(ray.o);
    if ray.o.x * ray.o.x + ray.o.y * ray.o.y <= 0.8 {
        (0.7, 0.2, 0.2)
    } else {
        (0.1, 0.1, 0.1)
    }
}

#[derive(Debug, Clone)]
struct TileData {
    idx: u32,
    remaining_samples: u32,
    framebuffer: Vec<(f32, f32, f32)>,
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

fn render_tile(i: usize, mut remaining_samples: usize, render_buffer: Arc<RwLock<Vec<u32>>>, framebuffer: &mut Vec<(f32, f32, f32)>) -> u32 {
    let tile_width = (WIDTH / 4).min(MAX_TILE_WIDTH);
    let tile_height = (HEIGHT / 4).min(MAX_TILE_HEIGHT);

    let num_horiz_tiles = (WIDTH + tile_width - 1) / tile_width;
    let num_vert_tiles = (HEIGHT + tile_height - 1) / tile_height;
    let num_tiles = num_vert_tiles * num_horiz_tiles;

    let tile_x = i % num_horiz_tiles;
    let tile_y = i / num_horiz_tiles;

    let this_tile_width = WIDTH - ((num_horiz_tiles.max(1) - 1) * tile_width);
    let this_tile_height = HEIGHT - ((num_vert_tiles.max(1) - 1) * tile_height);

    let this_tile_pixels = this_tile_height * this_tile_width;

    let pixel_start_x = tile_x * tile_width;
    let pixel_start_y = tile_y * tile_height;

    let mut temp_buffer = vec![0; this_tile_pixels];
    let start_time = Instant::now();

    while remaining_samples > 0 && Instant::now().saturating_duration_since(start_time).as_secs_f32() <= 0.25 {
        let new_remaining_samples = remaining_samples.saturating_sub(SAMPLE_CHUNK_SIZE);
        let samples_this_iter = remaining_samples - new_remaining_samples;

        for (i, p) in temp_buffer.iter_mut().enumerate() {
            let (nr, ng, nb) = get_pixel_color(
                pixel_start_x + i % this_tile_width,
                pixel_start_y + i / this_tile_width,
                samples_this_iter,
            );

            let (or, og, ob) = framebuffer[i];
            let (ar, ag, ab) = (or + nr, og + ng, ob + nb);
            framebuffer[i] = (ar, ab, ag);

            let w2 = (TOTAL_SPP as f32) / ((TOTAL_SPP - new_remaining_samples) as f32) * 255.99;
            *p = rgb_to_u32((ar * w2).min(255.99) as u8, (ag * w2).min(255.99) as u8, (ab * w2).min(255.99) as u8);
        }

        remaining_samples = new_remaining_samples;
    }

    let mut render_buffer = render_buffer.write().unwrap();
    for i in 0..this_tile_height {
        let abs = (pixel_start_y + i) * WIDTH + pixel_start_x;
        render_buffer[abs..(abs + this_tile_width)]
            .copy_from_slice(&temp_buffer[(i * this_tile_width)..((i + 1) * this_tile_width)]);
    }

    remaining_samples as u32
}

fn main() {
    let buffer: Arc<RwLock<Vec<u32>>> = Arc::new(RwLock::new(vec![0; WIDTH * HEIGHT]));

    let mut window =
        Window::new("Iris", WIDTH, HEIGHT, WindowOptions { resize: false, ..Default::default() }).unwrap_or_else(|e| {
            panic!("failed to create window: {}", e);
        });

    let mut update_rate = frame::UpdateRate::new(std::time::Duration::from_micros(16600)); // 60fps
    window.limit_update_rate(None);

    let buffer2 = buffer.clone();

    let tile_width = (WIDTH / num_cpus::get()).min(MAX_TILE_WIDTH);
    let tile_height = (HEIGHT / num_cpus::get()).min(MAX_TILE_HEIGHT);

    let num_horiz_tiles = (WIDTH + tile_width - 1) / tile_width;
    let num_vert_tiles = (HEIGHT + tile_height - 1) / tile_height;
    let num_tiles = num_vert_tiles * num_horiz_tiles;

    let tile_priorities = Arc::new(Mutex::new((0..num_tiles)
        .map(|idx| TileData {
            idx: idx as u32,
            remaining_samples: TOTAL_SPP as u32,
            framebuffer: vec![(0.0, 0.0, 0.0); tile_width * tile_height],
        })
        .collect::<BinaryHeap<TileData>>()));

    for _cpu in 0..num_cpus::get() {
        let tile_priorities = tile_priorities.clone();
        let buffer = buffer.clone();
        std::thread::spawn(move || {
            loop {
                let popped = tile_priorities.lock().unwrap().pop();
                if let Some(TileData { idx, remaining_samples, mut framebuffer }) = popped {
                    let remaining_samples = render_tile(idx as usize, remaining_samples as usize, buffer.clone(), &mut framebuffer);

                    if remaining_samples > 0 {
                        tile_priorities.lock().unwrap().push(TileData { idx, remaining_samples, framebuffer });
                    }
                }
            }
        });
    }

    let mut done = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !done {
            if let Ok(tp) = tile_priorities.try_lock() {
                if tp.len() == 0 {
                    println!("Done!");
                    done = true;
                }
            }
        }

        update_rate.wait();
        let buffer = buffer2.read().unwrap();
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .expect("failed to update window buffer with pixel data");
    }
}
