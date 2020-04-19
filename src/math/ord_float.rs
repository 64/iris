#[derive(PartialEq, PartialOrd)]
pub struct OrdFloat(f32);

impl OrdFloat {
    pub fn new(f: f32) -> Self {
        // TODO: Benchmark unchecked variant?
        assert!(!f.is_nan());
        Self(f)
    }
}

impl std::cmp::Eq for OrdFloat {}

impl std::cmp::Ord for OrdFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other)
            .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() })
    }
}
