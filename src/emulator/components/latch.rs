#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    OFF = 0,
    ON = 1,
}

pub struct Latch {
    state: State,
}

pub fn new() -> Latch {
    Latch { state: State::OFF }
}

impl Latch {
    pub fn toggle(&mut self) {
        self.state = match self.state {
            State::OFF => State::ON,
            State::ON => State::OFF,
        }
    }

    pub fn get(&self) -> State {
        self.state
    }

    pub fn reset(&mut self) {
        self.state = State::OFF;
    }

    pub fn as_bool(&self) -> bool {
        self.state == State::ON
    }

    pub fn load_bool(&mut self, on: bool) {
        self.state = if on { State::ON } else { State::OFF };
    }
}
