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
    delegate: Rc<RefCell<dyn Ticker>>,
    factor: u32,
}

impl ScaledTicker {
    pub fn new(delegate: Rc<RefCell<dyn Ticker>>, factor: u32) -> ScaledTicker {
        ScaledTicker { delegate, factor }
    }
}

impl Ticker for ScaledTicker {
    fn tick(&mut self) -> u32 {
        self.delegate.borrow_mut().tick() * self.factor
    }
}

pub struct Clock {
    cycle_duration_ns: u64,
    elapsed_cycles: u64,
    pause_threshold_ns: u64,
    started_instant: Instant,
    tickers: Vec<Rc<RefCell<dyn Ticker>>>,
    turn_order: BinaryHeap<TickNode>,
}

impl Clock {
    pub fn new(cycle_duration_ns: u64, pause_threshold_ns: u64) -> Clock {
        Clock {
            cycle_duration_ns: cycle_duration_ns,
            elapsed_cycles: 0,
            pause_threshold_ns: pause_threshold_ns,
            started_instant: Instant::now(),
            tickers: Vec::new(),
            turn_order: BinaryHeap::new(),
        }
    }

    pub fn tick(&mut self) {
        let next_node = self.turn_order.pop();
        match next_node {
            Some(mut node) => {
                self.elapsed_cycles = node.next_tick_cycle;
                let cycles = self.tickers[node.ticker_ix].borrow_mut().tick();
                node.next_tick_cycle = self.elapsed_cycles + (cycles as u64);
                self.turn_order.push(node);
            },
            None => ()
        }

        let running_time = self.started_instant.elapsed();
        let running_time_ns = running_time.as_secs() * 1_000_000_000 + (running_time.subsec_nanos() as u64);
        let drift_ns = (self.elapsed_cycles * self.cycle_duration_ns).saturating_sub(running_time_ns);
        if drift_ns > self.pause_threshold_ns {
            thread::sleep(Duration::from_nanos(drift_ns));
        }
    }

    pub fn manage(&mut self, ticker: Rc<RefCell<Ticker>>) {
        self.tickers.push(ticker);
        let node = TickNode {
            ticker_ix: self.tickers.len() - 1,
            next_tick_cycle: self.elapsed_cycles,
        };
        self.turn_order.push(node);
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
        clock.manage(ticker.clone());

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
        let scaled_ticker3 = Rc::new(RefCell::new(ScaledTicker::new(ticker3.clone(), 3)));

        clock.manage(ticker1.clone());
        clock.manage(scaled_ticker3.clone());

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
