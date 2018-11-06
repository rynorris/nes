
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
    pub enabled: bool,
    pub period: u16,
    timer: u16,
    pub length: u8,
    pub halt_length: bool,
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
            enabled: false,
            period: 0,
            length: 0,
            halt_length: false,
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
        if !self.halt_length {
            self.length = self.length.saturating_sub(1);
        }
    }

    pub fn volume(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

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

pub struct Triangle {
    pub enabled: bool,
    pub period: u16,
    timer: u16,
    linear: u8,
    pub length: u8,
    pub halt_length: bool,
    pub linear_reload_flag: bool,
    pub linear_reload_value: u8,
    pub control_flag: bool,
    sequence_ix: u8,
}

impl Triangle {
    const SEQUENCE: [u8; 32] = [
        15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
         0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
    ];

    pub fn new() -> Triangle {
        Triangle {
            enabled: false,
            period: 0,
            timer: 0,
            linear: 0,
            length: 0,
            halt_length: false,
            linear_reload_flag: false,
            linear_reload_value: 0,
            control_flag: false,
            sequence_ix: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.timer == 0 {
            self.sequence_ix = (self.sequence_ix + 1) % 32;
            self.timer = self.period;
        } else {
            // Triangle timer clocks twice as fast as the other components.
            self.timer = self.timer.saturating_sub(2);
        }
    }

    pub fn clock_linear(&mut self) {
        if self.linear_reload_flag {
            self.linear = self.linear_reload_value;
        } else if self.linear > 0 {
            self.linear -= 1;
        }

        if !self.control_flag {
            self.linear_reload_flag = false;
        }
    }

    pub fn clock_length(&mut self) {
        if !self.halt_length {
            self.length = self.length.saturating_sub(1);
        }
    }

    pub fn volume(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

        if self.linear == 0 || self.length == 0 {
            return 0;
        }

        Triangle::SEQUENCE[self.sequence_ix as usize]
    }
}

pub struct Noise {
    pub enabled: bool,
    pub envelope: Envelope,
    shift_register: u16,
    pub length: u8,
    pub halt_length: bool,
    pub mode: bool,
    pub period: u16,
    timer: u16,
}

impl Noise {
    pub const PERIOD_LOOKUP: [u16; 16] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    ];

    pub fn new() -> Noise {
        Noise {
            enabled: false,
            envelope: Envelope::new(),
            shift_register: 1,
            length: 0,
            halt_length: false,
            mode: false,
            period: 0,
            timer: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            let bit1 = self.shift_register & 0x1;
            let bit2 = if self.mode {
                (self.shift_register >> 5) & 0x1
            } else {
                (self.shift_register >> 1) & 0x1
            };
            let feedback = bit1 ^ bit2;
            self.shift_register >>= 1;
            self.shift_register |= feedback << 13;
        } else {
            self.timer = self.timer.saturating_sub(1);
        }
    }

    pub fn clock_length(&mut self) {
        if !self.halt_length {
            self.length = self.length.saturating_sub(1);
        }
    }

    pub fn volume(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

        if self.shift_register & 0x1 != 0 {
            return 0;
        }

        if self.length == 0 {
            return 0;
        }

        self.envelope.volume()
    }
}
