
use emulator::clock::Ticker;
use emulator::memory::{Reader, Writer};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum SequenceMode {
    FourStep,
    FiveStep,
}

const LENGTH_COUNTER_LOOKUP: [u8; 0x20] = [
    10,254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct APU {
    sequence_mode: SequenceMode,
    cycle_counter: u64,
    irq_flag: bool,

    // Registers.
    pulse_1_timer: u8,
    pulse_1_length: u8,
    pulse_1_envelope: u8,
    pulse_1_sweep: u8,

    pulse_2_timer: u8,
    pulse_2_length: u8,
    pulse_2_envelope: u8,
    pulse_2_sweep: u8,

    triangle_timer: u8,
    triangle_length: u8,
    triangle_linear: u8,

    noise_timer: u8,
    noise_length: u8,
    noise_envelope: u8,
    noise_feedback: u8,
}

impl APU {
    pub fn new() -> APU {
        APU {
            sequence_mode: SequenceMode::FourStep,
            cycle_counter: 0,
            irq_flag: false,
            pulse_1_timer: 0,
            pulse_1_length: 0,
            pulse_1_envelope: 0,
            pulse_1_sweep: 0,

            pulse_2_timer: 0,
            pulse_2_length: 0,
            pulse_2_envelope: 0,
            pulse_2_sweep: 0,

            triangle_timer: 0,
            triangle_length: 0,
            triangle_linear: 0,

            noise_timer: 0,
            noise_length: 0,
            noise_envelope: 0,
            noise_feedback: 0,
        }
    }

    pub fn irq_triggered(&mut self) -> bool {
        self.irq_flag
    }

    fn clock_linear_counters(&mut self) {
        self.triangle_linear = self.triangle_linear.saturating_sub(1);
    }

    fn clock_length_counters(&mut self) {
        self.pulse_1_length = self.pulse_1_length.saturating_sub(1);
        self.pulse_2_length = self.pulse_2_length.saturating_sub(1);
        self.triangle_length = self.triangle_length.saturating_sub(1);
        self.noise_length = self.noise_length.saturating_sub(1);
    }

    fn addr_to_register(&mut self, address: u16) -> &mut u8 {
        match address {
            0x4000 => &mut self.pulse_1_timer,
            0x4001 => &mut self.pulse_1_length,
            0x4002 => &mut self.pulse_1_envelope,
            0x4003 => &mut self.pulse_1_sweep,

            0x4004 => &mut self.pulse_2_timer,
            0x4005 => &mut self.pulse_2_length,
            0x4006 => &mut self.pulse_2_envelope,
            0x4007 => &mut self.pulse_2_sweep,

            0x4008 => &mut self.triangle_timer,
            0x4009 => &mut self.triangle_length,
            0x400A => &mut self.triangle_linear,
            0x400B => &mut self.pulse_2_sweep,

            _ => panic!("Unexpected address in APU: ${:X}", address),
        }
    }
}

impl Ticker for APU {
    fn tick(&mut self) -> u32 {
        self.cycle_counter += 1;
        match self.sequence_mode {
            SequenceMode::FourStep => match self.cycle_counter {
                3729 => self.clock_linear_counters(),
                7457 => {
                    self.clock_linear_counters();
                    self.clock_length_counters();
                },
                11186 => self.clock_linear_counters(),
                14915 => {
                    self.clock_linear_counters();
                    self.clock_length_counters();
                    self.cycle_counter = 0;
                    self.irq_flag = true;
                },
                _ => (),
            },
            SequenceMode::FiveStep => match self.cycle_counter {
                3729 => self.clock_linear_counters(),
                7457 => {
                    self.clock_linear_counters();
                    self.clock_length_counters();
                },
                11186 => self.clock_linear_counters(),
                18641 => {
                    self.clock_linear_counters();
                    self.clock_length_counters();
                    self.cycle_counter = 0;
                },
                _ => (),
            },
        };
        1
    }
}

impl Writer for APU {
    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x4003 => {
                self.pulse_1_length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
            },
            0x4007 => {
                self.pulse_2_length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
            },
            0x400B => {
                self.triangle_length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
            },
            0x400F => {
                self.noise_length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
            },
            _ => (),
        }
    }
}

impl Reader for APU {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x4015 => {
                let mut status = 0;
                if self.pulse_1_length != 0 { status |= 1 };
                if self.pulse_2_length != 0 { status |= 1 << 1 };
                if self.triangle_length != 0 { status |= 1 << 2 };
                if self.noise_length != 0 { status |= 1 << 3 };
                if self.irq_flag { status |= 1 << 6 };

                self.irq_flag = false;
                status
            },
            _ => 0,
        }
    }
}
