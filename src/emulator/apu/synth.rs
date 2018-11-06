
pub struct Divider {
    period: u8,
    counter: u8,
}

impl Divider {
    pub fn new(period: u8) -> Divider {
        Divider {
            period,
            counter: 0,
        }
    }

    pub fn clock(&mut self) -> bool {
        if self.counter == 0 {
            self.counter = self.period;
            true
        } else {
            self.counter -= 1;
            false
        }
    }

    pub fn reload(&mut self) {
        self.counter = self.period;
    }

    pub fn set_period(&mut self, period: u8) {
        self.period = period;
    }
}

pub struct Envelope {
    start_flag: bool,
    decay_level: u8,
    divider: Divider,

    pub loop_flag: bool,
    pub constant_volume: bool,
    volume: u8,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            start_flag: true,
            decay_level: 0,
            divider: Divider::new(0),

            loop_flag: false,
            constant_volume: false,
            volume: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.start_flag {
            self.start_flag = false;
            self.decay_level = 15;
            self.divider.reload();
        } else {
            if self.divider.clock() {
                if self.decay_level == 0 {
                    if self.loop_flag { self.decay_level = 15 }
                } else {
                    self.decay_level -= 1;
                }
            }
        }
    }

    pub fn volume(&self) -> u8 {
        if self.constant_volume {
            self.volume
        } else {
            self.decay_level
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume;
        self.divider.set_period(volume);
    }

    pub fn restart(&mut self) {
        self.start_flag = true;
    }
}

pub struct Pulse {
    pub period: u16,
    timer: u16,
    pub length: u8,
    pub sequence: u8,
    sequence_ix: u8,
    pub envelope: Envelope,
}

impl Pulse {
    const SEQUENCES: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 0, 0],
    ];

    pub fn new() -> Pulse {
        Pulse {
            period: 0,
            length: 0,
            sequence: 0,
            envelope: Envelope::new(),
            timer: 0,
            sequence_ix: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            self.sequence_ix = if self.sequence_ix == 0 { 7 } else { self.sequence_ix - 1 }
        } else {
            self.timer -= 1;
        }
    }

    pub fn clock_length(&mut self) {
        self.length = self.length.saturating_sub(1);
    }

    pub fn volume(&self) -> u8 {
        if self.timer < 8 {
            return 0;
        }

        if self.length == 0 {
            return 0;
        }

        if Pulse::SEQUENCES[self.sequence as usize][self.sequence_ix as usize] == 0 {
            return 0;
        }

        self.envelope.volume() 
    }

    pub fn restart(&mut self) {
        self.sequence_ix = 0;
        self.envelope.restart();
    }
}
