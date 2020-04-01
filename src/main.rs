#![feature(clamp)]
use minifb::{Key, Window, WindowOptions};

use std::{
    collections::BinaryHeap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex,
        RwLock,
    },
    time::{Duration, Instant},
};

mod color;
mod math;
mod sampler;
mod scene;
mod shapes;
mod spectrum;
mod tile;
mod upsample;

pub struct Render {
    pub width: usize,
    pub height: usize,
    pub spp: usize,
    pub buffer: RwLock<Vec<u32>>,
    pub scene: Scene,
}

use scene::Scene;
use tile::TileData;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const TOTAL_SPP: usize = 200;

static DONE: AtomicBool = AtomicBool::new(false);

fn main() {
    let render = Arc::new(Render {
        width: WIDTH,
        height: HEIGHT,
        spp: TOTAL_SPP,
        buffer: RwLock::new(vec![0; WIDTH * HEIGHT]),
        scene: scene::Scene::dummy(),
    });

    let mut window = Window::new(
        "Iris",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..Default::default()
        },
    )
    .expect("failed to create window");

    let tile_priorities = Arc::new(Mutex::new(
        // TODO: Make this nice
        (0..)
            .map(|idx| TileData::new(&render, idx))
            .take_while(|t| t.is_some())
            .map(|t| t.unwrap())
            .collect::<BinaryHeap<TileData>>(),
    ));

    for _cpu in 0..num_cpus::get() {
        let tile_priorities = tile_priorities.clone();
        let render = render.clone();
        std::thread::spawn(move || loop {
            let popped = tile_priorities.lock().unwrap().pop();
            match popped {
                Some(tile) => {
                    let tile = tile.render(&render);
                    if tile.remaining_samples > 0 {
                        tile_priorities.lock().unwrap().push(tile);
                    }
                }
                None => {
                    if !DONE.swap(true, Ordering::Relaxed) {
                        println!("Done!");
                    }
                    break;
                }
            }
        });
    }

    let target_rate = std::time::Duration::from_micros(33333); // 30fps
    window.limit_update_rate(None);

    let mut prev_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // TODO: Use some kind of sleeping mutex so that we only update the screen when
        // the render buffer has changed?

        // Wait for 30fps
        let target_rate = target_rate.as_secs_f64();
        let current_time = Instant::now();
        let delta = current_time
            .saturating_duration_since(prev_time)
            .as_secs_f64();

        if delta < target_rate {
            let sleep_time = target_rate - delta;
            if sleep_time > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(sleep_time));
            }
        }

        prev_time = Instant::now();

        let buffer = render.buffer.read().unwrap();
        window
            .update_with_buffer(&buffer, render.width, render.height)
            .expect("failed to update window buffer with pixel data");
    }
}
