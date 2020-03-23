use minifb::{Key, Window, WindowOptions};

use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Instant;

mod math;
mod frame;
mod tile;

use math::{Point3, Ray, Vec3};
use tile::TileData;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

const MAX_TILE_WIDTH: usize = 64;
const MAX_TILE_HEIGHT: usize = 64;
const TOTAL_SPP: usize = 500;
const SAMPLE_CHUNK_SIZE: usize = 20;

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
            idx,
            remaining_samples: TOTAL_SPP,
            framebuffer: vec![(0.0, 0.0, 0.0); tile_width * tile_height],
        })
        .collect::<BinaryHeap<TileData>>()));

    for _cpu in 0..num_cpus::get() {
        let tile_priorities = tile_priorities.clone();
        let buffer = buffer.clone();
        std::thread::spawn(move || {
            loop {
                let popped = tile_priorities.lock().unwrap().pop();
                if let Some(tile) = popped {
                    let tile = tile.render(&buffer);
                    if tile.remaining_samples > 0 {
                        tile_priorities.lock().unwrap().push(tile);
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
