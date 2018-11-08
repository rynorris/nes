use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::rc::Rc;
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
    // Timing.
    elapsed_cycles: u64,

    // Tickers.
    tickers: Vec<ScaledTicker>,
    turn_order: BinaryHeap<TickNode>,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            elapsed_cycles: 0,
            tickers: Vec::new(),
            turn_order: BinaryHeap::new(),
        }
    }

    #[inline]
    pub fn tick(&mut self) -> u64 {
        match self.turn_order.peek_mut() {
            Some(mut node) => {
                let cycles_waited = node.next_tick_cycle - self.elapsed_cycles;
                self.elapsed_cycles = node.next_tick_cycle;
                let cycles = self.tickers[node.ticker_ix].tick();
                node.next_tick_cycle = self.elapsed_cycles + (cycles as u64);
                cycles_waited
            },
            None => 0
        }
    }

    pub fn manage(&mut self, ticker: ScaledTicker) {
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
        let mut clock = Clock::new();
        let ticker = Rc::new(RefCell::new(DummyTicker::new()));
        clock.manage(ScaledTicker::new(Box::new(ticker.clone()), 1));

        clock.tick();
        assert_eq!(ticker.borrow().value, 1);
        clock.tick();
        assert_eq!(ticker.borrow().value, 2);
        clock.tick();
        assert_eq!(ticker.borrow().value, 3);
    }

    #[test]
    fn test_scaled_ticker() {
        let mut clock = Clock::new();
        let ticker1 = Rc::new(RefCell::new(DummyTicker::new()));
        let ticker3 = Rc::new(RefCell::new(DummyTicker::new()));

        clock.manage(ScaledTicker::new(Box::new(ticker1.clone()), 1));
        clock.manage(ScaledTicker::new(Box::new(ticker3.clone()), 3));

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
