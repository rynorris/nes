use std::thread;
use std::time::{Duration, Instant};

pub struct Governer {
    target_frame_ns: u64,
    frame_start_instant: Instant,
    ahead_ns: i64,
    frame_duration_mavg: MovingAverage,
}

impl Governer {
    const MIN_BUFFER_NS: i64 = -3_000_000;
    const MAX_BUFFER_NS: i64 = 3_000_000;

    pub fn new(target_fps: u64) -> Governer {
        Governer {
            target_frame_ns: 1_000_000_000 / target_fps,
            frame_start_instant: Instant::now(),
            ahead_ns: 0,
            frame_duration_mavg: MovingAverage::new(target_fps as usize),
        }
    }

    pub fn taking_too_long(&self) -> bool {
        let frame_ns = duration_to_ns(self.frame_start_instant.elapsed());
        return frame_ns > self.target_frame_ns + (self.ahead_ns as u64);
    }

    pub fn synchronize(&mut self) {
        let frame_ns = duration_to_ns(self.frame_start_instant.elapsed());
        self.ahead_ns += self.target_frame_ns as i64;
        self.ahead_ns -= frame_ns as i64;
        if self.ahead_ns < Governer::MIN_BUFFER_NS {
            self.ahead_ns = Governer::MIN_BUFFER_NS;
        }

        if self.ahead_ns > Governer::MAX_BUFFER_NS {
            thread::sleep(Duration::from_nanos(self.ahead_ns as u64));
        }

        let frame_end_instant = Instant::now();
        let total_frame_ns = duration_to_ns(frame_end_instant - self.frame_start_instant);
        let sleep_ns = total_frame_ns.saturating_sub(frame_ns);
        self.ahead_ns -= sleep_ns as i64;

        self.frame_start_instant = frame_end_instant;
        self.frame_duration_mavg.update(total_frame_ns as f64);
    }

    pub fn avg_frame_duration_ns(&self) -> f64 {
        self.frame_duration_mavg.get()
    }
}

fn duration_to_ns(duration: Duration) -> u64 {
    duration.as_secs() * 1_000_000_000 + (duration.subsec_nanos() as u64)
}

pub struct MovingAverage {
    num_samples: usize,
    samples: Vec<f64>,
    sum: f64,
}

impl MovingAverage {
    pub fn new(num_samples: usize) -> MovingAverage {
        MovingAverage {
            num_samples,
            samples: Vec::with_capacity(num_samples),
            sum: 0f64,
        }
    }

    pub fn get(&self) -> f64 {
        self.sum / (self.num_samples as f64)
    }

    pub fn update(&mut self, sample: f64) {
        self.sum += sample;
        self.samples.push(sample);
        if self.samples.len() > self.num_samples {
            self.sum -= self.samples.remove(0);
        }
    }
}
