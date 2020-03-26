use minifb::{Key, Window, WindowOptions};

use std::{
    collections::BinaryHeap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex,
        RwLock,
    },
};

mod frame;
mod math;
mod render;
mod spectrum;
mod tile;
mod color;
mod sampler;

use render::Render;
use tile::TileData;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const TOTAL_SPP: usize = 1000;

static DONE: AtomicBool = AtomicBool::new(false);

fn main() {
    let render = Arc::new(Render {
        width: WIDTH,
        height: HEIGHT,
        spp: TOTAL_SPP,
        buffer: RwLock::new(vec![0; WIDTH * HEIGHT]),
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

    let mut update_rate = frame::UpdateRate::new(std::time::Duration::from_micros(16600)); // 60fps
    window.limit_update_rate(None);

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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        update_rate.wait();
        let buffer = render.buffer.read().unwrap();
        window
            .update_with_buffer(&buffer, render.width, render.height)
            .expect("failed to update window buffer with pixel data");
    }
}
