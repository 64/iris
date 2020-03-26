use std::sync::RwLock;

pub struct Render {
    pub width: usize,
    pub height: usize,
    pub spp: usize,
    pub buffer: RwLock<Vec<u32>>,
}
