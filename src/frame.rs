use std::time::{Duration, Instant};

// Adapted from https://github.com/emoon/rust_minifb
pub struct UpdateRate {
    target_rate: Duration,
    prev_time: Instant,
}

impl UpdateRate {
    pub fn new(target_rate: Duration) -> UpdateRate {
        UpdateRate {
            target_rate,
            prev_time: Instant::now(),
        }
    }

    pub fn wait(&mut self) {
        let target_rate = self.target_rate.as_secs_f64();
        let current_time = Instant::now();
        let delta = current_time
            .saturating_duration_since(self.prev_time)
            .as_secs_f64();

        if delta < target_rate {
            let sleep_time = target_rate - delta;
            if sleep_time > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(sleep_time));
            }
        }

        self.prev_time = Instant::now();
    }
}
