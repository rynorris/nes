use std::thread;
use std::time::{Duration, Instant};

pub struct Governer {
    target_frame_ns: u64,
    frame_start_instant: Instant,
    ahead_ns: i64,
}

impl Governer {
    const MIN_BUFFER_NS: i64 = -3_000_000;
    const MAX_BUFFER_NS: i64 = 3_000_000;

    pub fn new(target_fps: u64) -> Governer {
        Governer {
            target_frame_ns: 1_000_000_000 / target_fps,
            frame_start_instant: Instant::now(),
            ahead_ns: 0,
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
    }
}

fn duration_to_ns(duration: Duration) -> u64 {
    duration.as_secs() * 1_000_000_000 + (duration.subsec_nanos() as u64)
}
