use emulator::memory::Reader;

pub struct Divider {
    period: u16,
    counter: u16,
}

impl Divider {
    pub fn new(period: u16) -> Divider {
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

    pub fn set_period(&mut self, period: u16) {
        self.period = period;
    }

    pub fn period(&self) -> u16 {
        self.period
    }

    pub fn counter(&self) -> u16 {
        self.counter
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
        self.divider.set_period(volume as u16);
    }

    pub fn restart(&mut self) {
        self.start_flag = true;
    }
}

pub struct Sweep {
    twos_complement: bool,
    pub enabled: bool,
    pub divider: Divider,
    pub negate_flag: bool,
    pub shift_count: u8,
    reload_flag: bool,
    target_period: u16,
}

impl Sweep {
    pub fn new(twos_complement: bool) -> Sweep {
        Sweep {
            twos_complement,
            enabled: true,
            divider: Divider::new(0),
            negate_flag: false,
            shift_count: 0,
            reload_flag: false,
            target_period: 0,
        }
    }

    pub fn clock(&mut self, current_period: u16) {
        let change_amount = current_period >> self.shift_count;
        self.target_period = if self.negate_flag {
            if self.twos_complement {
                current_period.saturating_sub(change_amount + 1)
            } else {
                current_period.saturating_sub(change_amount)
            }
        } else {
            current_period.saturating_add(change_amount)
        };
    }

    pub fn get_updated_period(&mut self, current_period: u16) -> u16 {
        if self.divider.clock() {
            self.reload_flag = false;
            if self.shift_count != 0 && self.enabled && !self.is_muting(current_period) {
                return self.target_period;
            }
        }

        if self.reload_flag {
            self.divider.reload();
            self.reload_flag = false;
        }

        current_period
    }

    pub fn is_muting(&self, current_period: u16) -> bool {
        self.target_period > 0x7FF || current_period < 8
    }
}

pub struct Pulse {
    pub enabled: bool,
    pub timer: Divider,
    pub length: u8,
    pub halt_length: bool,
    pub sequence: u8,
    sequence_ix: u8,
    pub envelope: Envelope,
    pub sweep: Sweep,
}

impl Pulse {
    const SEQUENCES: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 0, 0],
    ];

    pub fn new(sweep: Sweep) -> Pulse {
        Pulse {
            enabled: false,
            timer: Divider::new(0),
            length: 0,
            halt_length: false,
            sequence: 0,
            envelope: Envelope::new(),
            sweep,
            sequence_ix: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.timer.clock() {
            self.sequence_ix = if self.sequence_ix == 0 { 7 } else { self.sequence_ix - 1 }
        }

        self.sweep.clock(self.timer.period());
    }

    pub fn clock_length(&mut self) {
        if !self.halt_length {
            self.length = self.length.saturating_sub(1);
        }

        let new_period = self.sweep.get_updated_period(self.timer.period());
        self.timer.set_period(new_period);
    }

    pub fn volume(&self) -> u8 {
        if !self.enabled {
            return 0;
        }

        if self.timer.counter() < 8 {
            return 0;
        }

        if self.length == 0 {
            return 0;
        }

        if Pulse::SEQUENCES[self.sequence as usize][self.sequence_ix as usize] == 0 {
            return 0;
        }

        if self.sweep.is_muting(self.timer.period()) {
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
    pub timer: Divider,
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
            timer: Divider::new(0),
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
        if self.timer.clock() {
            self.sequence_ix = (self.sequence_ix + 1) % 32;
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
    pub timer: Divider,
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
            timer: Divider::new(0),
        }
    }

    pub fn clock(&mut self) {
        if self.timer.clock() {
            let bit1 = self.shift_register & 0x1;
            let bit2 = if self.mode {
                (self.shift_register >> 5) & 0x1
            } else {
                (self.shift_register >> 1) & 0x1
            };
            let feedback = bit1 ^ bit2;
            self.shift_register >>= 1;
            self.shift_register |= feedback << 13;
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

pub struct DMC {
    // Config.
    pub enabled: bool,
    pub irq_enabled: bool,
    pub loop_flag: bool,
    silence_flag: bool,
    pub timer: Divider,
    pub volume: u8,
    pub sample_addr: u16,
    pub sample_len: u16,

    // State.
    prg_rom: Box<dyn Reader>,  // Read-only access to ROM.
    sample_buffer: Option<u8>,
    current_addr: u16,
    pub bytes_remaining: u16,
    pub irq_flag: bool,

    shift_register: u8,
    bits_remaining: u8,
}

impl DMC {
    pub const PERIOD_LOOKUP: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

    pub fn new(prg_rom: Box<dyn Reader>) -> DMC {
        DMC {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            silence_flag: false,
            timer: Divider::new(0),
            volume: 0,
            sample_addr: 0,
            sample_len: 0,

            prg_rom,
            sample_buffer: None,
            current_addr: 0,
            bytes_remaining: 0,
            irq_flag: false,

            shift_register: 0,
            bits_remaining: 0,
        }
    }

    pub fn clock(&mut self) {
        if self.timer.clock() {
            self.clock_memory_reader();
            self.clock_output_unit();
        }
    }

    pub fn restart_sample(&mut self) {
        self.bytes_remaining = self.sample_len;
        self.current_addr = self.sample_addr;
    }

    fn clock_memory_reader(&mut self) {
        if self.sample_buffer.is_none() && self.bytes_remaining != 0 {
            // TODO: Stall the CPU by 4 (or 2) clocks.
            let byte = self.prg_rom.read(self.current_addr);
            self.sample_buffer = Some(byte);
            self.current_addr = self.current_addr.wrapping_add(1);
            if self.current_addr == 0 { self.current_addr = 0x8000 };

            self.bytes_remaining = self.bytes_remaining.saturating_sub(1);
            if self.bytes_remaining == 0 {
                if self.loop_flag {
                    self.restart_sample();
                } else if self.irq_enabled {
                    self.irq_flag = true;
                }
            }
        }
    }

    fn clock_output_unit(&mut self) {
        if !self.silence_flag {
            let bit = self.shift_register & 0x01;
            if bit == 1 && self.volume <= 125 {
                self.volume += 2;
            } else if bit == 0 && self.volume >= 2 {
                self.volume -= 2;
            }
        }

        self.shift_register >>= 1;
        self.bits_remaining = self.bits_remaining.saturating_sub(1);

        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            if self.sample_buffer.is_none() {
                self.silence_flag = true;
            } else {
                self.silence_flag = false;
                self.shift_register = self.sample_buffer.take().unwrap();
            }
        }
    }
}
