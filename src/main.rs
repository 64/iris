#![feature(portable_simd)]

extern crate sobol_burley as sobol;

use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};

mod bsdf;
mod camera;
mod color;
mod integrator;
mod math;
mod sampling;
mod scene;
mod shape;
mod spectrum;
mod tile;
mod types;

use camera::Camera;
use scene::Scene;
use tile::TileData;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const TOTAL_SPP: usize = 100;

type CurrentIntegrator = integrator::hwss_naive::HwssNaive;

pub struct Render {
    pub width: usize,
    pub height: usize,
    pub spp: usize,
    pub scene: Scene,
    pub camera: Camera,
    pub buffer: RwLock<Vec<(f32, f32, f32)>>,
    pub integrator: CurrentIntegrator,
}

fn main() {
    let render = Arc::new(Render {
        width: WIDTH,
        height: HEIGHT,
        spp: TOTAL_SPP,
        integrator: CurrentIntegrator::default(),
        scene: scene::Scene::dummy(),
        buffer: RwLock::new(vec![(0.0, 0.0, 0.0); WIDTH * HEIGHT]),
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

    let num_threads = std::env::var("NTHREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(num_cpus::get);

    do_render(render, tile_priorities, num_threads);
}

#[cfg(not(feature = "progressive"))]
fn do_render(
    render: Arc<Render>,
    tile_priorities: Arc<Mutex<BinaryHeap<TileData>>>,
    num_threads: usize,
) {
    println!(
        "Starting render, {}x{}@{}spp...",
        render.width, render.height, render.spp
    );

    let start = Instant::now();

    let threads = (0..num_threads)
        .map(|_| {
            let tile_priorities = tile_priorities.clone();
            let render = render.clone();
            std::thread::spawn(move || loop {
                let popped = tile_priorities.lock().unwrap().pop();
                match popped {
                    Some(tile) => {
                        tile.render(&render);
                    }
                    None => {
                        break;
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().unwrap();
    }

    let elapsed = start.elapsed().as_secs_f32();
    println!(
        "Done in {}s ({}m ray/s)",
        elapsed,
        ((render.spp * WIDTH * HEIGHT) as f32) / (1_000_000.0 * elapsed),
    );

    use exr::prelude::*;

    let buffer = render.buffer.read().unwrap();

    write_rgb_file("out.exr", render.width, render.height, |x, y| {
        buffer[x + y * render.width]
    })
    .unwrap();
}

#[cfg(feature = "progressive")]
fn do_render(
    render: Arc<Render>,
    tile_priorities: Arc<Mutex<BinaryHeap<TileData>>>,
    num_threads: usize,
) {
    use std::{
        io::Write,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    };

    use minifb::{Key, Window, WindowOptions};

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

    let samples_taken = Arc::new(AtomicUsize::new(0));
    static DONE: AtomicBool = AtomicBool::new(false);

    println!("Starting render...");

    let start = Instant::now();

    for _ in 0..num_threads {
        let tile_priorities = tile_priorities.clone();
        let render = render.clone();
        let samples_taken = samples_taken.clone();
        std::thread::spawn(move || loop {
            let popped = tile_priorities.lock().unwrap().pop();
            match popped {
                Some(tile) => {
                    let samples_before = tile.remaining_samples;
                    let tile = tile.render(&render);
                    let samples_after = tile.remaining_samples;
                    samples_taken.fetch_add(
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

    window.set_target_fps(10);

    let mut fb = vec![0u32; render.width * render.height];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !DONE.load(Ordering::Relaxed) {
            let progress = samples_taken.load(Ordering::Relaxed) as f32
                / (render.spp * render.width * render.height) as f32;
            print!("Progress: {:>5.2}%\r", 100.0 * progress);
            std::io::stdout().flush().unwrap();
        }

        let buffer = render.buffer.read().unwrap();

        for (i, pixel) in buffer.iter().enumerate() {
            fb[i] = ((pixel.0 as u32) << 16) | ((pixel.1 as u32) << 8) | (pixel.2 as u32);
            if *pixel != (0.0, 0.0, 0.0) {
                dbg!(pixel);
            }
        }

        window
            .update_with_buffer(&fb, render.width, render.height)
            .expect("failed to update window buffer with pixel data");
    }
}
