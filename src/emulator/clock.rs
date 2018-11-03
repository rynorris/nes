use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::thread;
use std::vec::Vec;

pub trait Ticker {
    // Returns how many master clock cycles while ticking.
    fn tick(&mut self) -> u32;
}

pub struct ScaledTicker {
    delegate: Box<dyn Ticker>,
    factor: u32,
}

impl ScaledTicker {
    pub fn new(delegate: Box<dyn Ticker>, factor: u32) -> ScaledTicker {
        ScaledTicker { delegate, factor }
    }
}

impl Ticker for ScaledTicker {
    #[inline]
    fn tick(&mut self) -> u32 {
        self.delegate.tick() * self.factor
    }
}

impl <T : Ticker> Ticker for Rc<RefCell<T>> {
    #[inline]
    fn tick(&mut self) -> u32 {
        self.borrow_mut().tick()
    }
}

pub struct Clock {
    // Configuration.
    cycle_duration_ps: u64,
    pause_threshold_ns: u64,
    started_instant: Instant,
    last_sync_ns: u64,

    // Timing.
    num_ticks: u64,
    elapsed_cycles: u64,
    elapsed_seconds: u64,
    cycles_this_second: u64,
    cycles_since_sync: u64,

    // Tickers.
    tickers: Vec<Box<dyn Ticker>>,
    turn_order: BinaryHeap<TickNode>,
}

impl Clock {
    pub fn new(cycle_duration_ps: u64, pause_threshold_ns: u64) -> Clock {
        Clock {
            cycle_duration_ps: cycle_duration_ps,
            num_ticks: 0,
            elapsed_cycles: 0,
            elapsed_seconds: 0,
            cycles_this_second: 0,
            cycles_since_sync: 0,
            pause_threshold_ns: pause_threshold_ns,
            started_instant: Instant::now(),
            last_sync_ns: 0,
            tickers: Vec::new(),
            turn_order: BinaryHeap::new(),
        }
    }

    pub fn tick(&mut self) {
        match self.turn_order.peek_mut() {
            Some(mut node) => {
                self.cycles_this_second += node.next_tick_cycle - self.elapsed_cycles;
                self.cycles_since_sync += node.next_tick_cycle - self.elapsed_cycles;
                self.elapsed_cycles = node.next_tick_cycle;
                let cycles = self.tickers[node.ticker_ix].tick();
                node.next_tick_cycle = self.elapsed_cycles + (cycles as u64);
            },
            None => ()
        }

        self.num_ticks += 1;
    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.elapsed_seconds
    }

    pub fn manage(&mut self, ticker: Box<Ticker>) {
        self.tickers.push(ticker);
        let node = TickNode {
            ticker_ix: self.tickers.len() - 1,
            next_tick_cycle: self.elapsed_cycles,
        };
        self.turn_order.push(node);
    }

    pub fn synchronize(&mut self) {
        let elapsed_time = self.started_instant.elapsed();
        let time_ns = elapsed_time.as_secs() * 1_000_000_000 + (elapsed_time.subsec_nanos() as u64);
        let since_sync_ns = time_ns - self.last_sync_ns;
        let drift_ns = ((self.cycles_since_sync * self.cycle_duration_ps) / 1000).saturating_sub(since_sync_ns);
        if drift_ns > self.pause_threshold_ns {
            thread::sleep(Duration::from_nanos(drift_ns));
        }

        let elapsed_seconds = self.started_instant.elapsed().as_secs();
        if self.elapsed_seconds != elapsed_seconds {
            self.elapsed_seconds = elapsed_seconds;
            let nes_freq = 21.477f64;
            let target_freq = 1_000_000f64 / (self.cycle_duration_ps as f64);
            let actual_freq = (self.cycles_this_second as f64) / 1_000_000f64;
            println!("Target: {:.3}MHz,  Current: {:.3}MHz ({:.2}x).",
                 target_freq,
                 actual_freq,
                 actual_freq / nes_freq,
            );
            self.cycles_this_second = 0;
        }
    }

    pub fn set_master_clock(&mut self, duration_ps: u64) {
        if duration_ps == self.cycle_duration_ps {
            return;
        }

        self.cycle_duration_ps = duration_ps;

        // Reset sync state, because changing the clock speed screws it all up.
        let elapsed_time = self.started_instant.elapsed();
        let time_ns = elapsed_time.as_secs() * 1_000_000_000 + (elapsed_time.subsec_nanos() as u64);
        self.last_sync_ns = time_ns;
        self.cycles_since_sync = 0;

    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TickNode {
    ticker_ix: usize,
    next_tick_cycle: u64,
}

impl Ord for TickNode {
    fn cmp(&self, other: &TickNode) -> Ordering {
        // Flip the ordering here to create a min-heap.
        other.next_tick_cycle.cmp(&self.next_tick_cycle)
    }
}

impl PartialOrd for TickNode {
    fn partial_cmp(&self, other: &TickNode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;

    use emulator::clock::{Clock, ScaledTicker, Ticker};

    struct DummyTicker {
        value: u16,
    }

    impl DummyTicker {
        fn new() -> DummyTicker {
            DummyTicker {
                value: 0,
            }
        }
    }

    impl Ticker for DummyTicker {
        fn tick(&mut self) -> u32 {
            self.value += 1;
            1
        }
    }

    #[test]
    fn test_single_ticker() {
        let mut clock = Clock::new(0, 1);
        let ticker = Rc::new(RefCell::new(DummyTicker::new()));
        clock.manage(Box::new(ticker.clone()));

        clock.tick();
        assert_eq!(ticker.borrow().value, 1);
        clock.tick();
        assert_eq!(ticker.borrow().value, 2);
        clock.tick();
        assert_eq!(ticker.borrow().value, 3);
    }

    #[test]
    fn test_scaled_ticker() {
        let mut clock = Clock::new(0, 1);
        let ticker1 = Rc::new(RefCell::new(DummyTicker::new()));
        let ticker3 = Rc::new(RefCell::new(DummyTicker::new()));
        let scaled_ticker3 = Rc::new(RefCell::new(ScaledTicker::new(Box::new(ticker3.clone()), 3)));

        clock.manage(Box::new(ticker1.clone()));
        clock.manage(Box::new(scaled_ticker3.clone()));

        // Tick twice first since the initial order is undefined.
        clock.tick();
        clock.tick();
        assert_eq!(ticker1.borrow().value, 1);
        assert_eq!(ticker3.borrow().value, 1);

        clock.tick();
        assert_eq!(ticker1.borrow().value, 2);
        assert_eq!(ticker3.borrow().value, 1);

        clock.tick();
        assert_eq!(ticker1.borrow().value, 3);
        assert_eq!(ticker3.borrow().value, 1);

        // And again here when their periods align.
        clock.tick();
        clock.tick();
        assert_eq!(ticker1.borrow().value, 4);
        assert_eq!(ticker3.borrow().value, 2);
    }
}
