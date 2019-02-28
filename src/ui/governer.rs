use std::thread;
use std::time::{Duration, Instant};

pub struct Governer {
    target_frame_ns: u64,
    target_ns_this_frame: u64,
    frame_start_instant: Instant,
}

impl Governer {
    pub fn new(target_fps: u64) -> Governer {
        Governer {
            target_frame_ns: 1_000_000_000 / target_fps,
            target_ns_this_frame: 1_000_000_000 / target_fps,
            frame_start_instant: Instant::now(),
        }
    }

    pub fn taking_too_long(&self) -> bool {
        let frame_ns = duration_to_ns(self.frame_start_instant.elapsed());
        return frame_ns > self.target_ns_this_frame;
    }

    pub fn synchronize(&mut self) {
        let frame_ns = duration_to_ns(self.frame_start_instant.elapsed());
        if frame_ns < self.target_ns_this_frame {
            let sleep_ns = self.target_frame_ns.saturating_sub(frame_ns);
            thread::sleep(Duration::from_nanos(sleep_ns));
        }

        let frame_end_instant = Instant::now();
        let total_frame_ns = (frame_end_instant - self.frame_start_instant).subsec_nanos() as u64;
        let oversleep_ns = total_frame_ns.saturating_sub(self.target_ns_this_frame);
        println!("Rendered frame in {:?}ns.  Overslept by {:?}ns", frame_ns, oversleep_ns);
        self.target_ns_this_frame = self.target_frame_ns.saturating_sub(oversleep_ns);
        self.frame_start_instant = frame_end_instant;
    }
}

fn duration_to_ns(duration: Duration) -> u64 {
    duration.as_secs() * 1_000_000_000 + (duration.subsec_nanos() as u64)
}
