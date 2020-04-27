#![feature(stdarch)]
#![feature(clamp)]
use minifb::{Key, Window, WindowOptions};

use std::{
    collections::BinaryHeap,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
        Mutex,
        RwLock,
    },
    time::{Duration, Instant},
};

mod bsdf;
mod camera;
mod color;
mod math;
mod sampling;
mod scene;
mod shapes;
mod spectrum;
mod tile;
mod types;

pub struct Render {
    pub width: usize,
    pub height: usize,
    pub spp: usize,
    pub buffer: RwLock<Vec<u32>>,
    pub scene: Scene,
    pub camera: Camera,
}

use camera::Camera;
use scene::Scene;
use tile::TileData;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const TOTAL_SPP: usize = 100;

static DONE: AtomicBool = AtomicBool::new(false);
static SAMPLES_TAKEN: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let render = Arc::new(Render {
        width: WIDTH,
        height: HEIGHT,
        spp: TOTAL_SPP,
        buffer: RwLock::new(vec![0; WIDTH * HEIGHT]),
        scene: scene::Scene::dummy(),
        camera: Camera::new(
            math::Point3::new(0.0, 0.0, 0.0),
            (WIDTH as f32) / (HEIGHT as f32),
        ),
    });

    let tile_priorities = Arc::new(Mutex::new(
        // TODO: Make this nice
        (0..)
            .map(|idx| TileData::new(&render, idx))
            .take_while(|t| t.is_some())
            .map(|t| t.unwrap())
            .collect::<BinaryHeap<TileData>>(),
    ));

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

    let num_threads = std::env::var("NTHREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(num_cpus::get);

    println!("Starting render...");

    let start = Instant::now();

    for _ in 0..num_threads {
        let tile_priorities = tile_priorities.clone();
        let render = render.clone();
        std::thread::spawn(move || loop {
            let popped = tile_priorities.lock().unwrap().pop();
            match popped {
                Some(tile) => {
                    let samples_before = tile.remaining_samples;
                    let tile = tile.render(&render);
                    let samples_after = tile.remaining_samples;
                    SAMPLES_TAKEN.fetch_add(
                        (samples_before - samples_after) * tile.width * tile.height,
                        Ordering::Relaxed,
                    );

                    if samples_after > 0 {
                        tile_priorities.lock().unwrap().push(tile);
                    }
                }
                None => {
                    if !DONE.swap(true, Ordering::Relaxed) {
                        let elapsed = start.elapsed().as_secs_f32();
                        println!(
                            "Done in {}s ({}m ray/s)",
                            elapsed,
                            ((render.spp * WIDTH * HEIGHT) as f32) / (1_000_000.0 * elapsed),
                        );
                    }
                    break;
                }
            }
        });
    }

    let target_rate = std::time::Duration::from_micros(100000); // 10fps
    window.limit_update_rate(None);

    let mut prev_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !DONE.load(Ordering::Relaxed) {
            let progress =
                SAMPLES_TAKEN.load(Ordering::Relaxed) as f32 / (render.spp * WIDTH * HEIGHT) as f32;
            print!("Progress: {:>5.2}%\r", 100.0 * progress);
            std::io::stdout().flush().unwrap();
        }

        // TODO: Use some kind of sleeping mutex so that we only update the screen when
        // the render buffer has changed?
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
