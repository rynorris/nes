#[derive(Clone, Copy)]
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
}
