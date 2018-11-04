
use emulator::clock::Ticker;
use emulator::memory::{Reader, Writer};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum SequenceMode {
    FourStep,
    FiveStep,
}

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

    pub fn irq_triggered(&self) -> bool {
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
        self.irq_flag = false;
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
                    self.irq_flag = true;
                },
                _ => (),
            },
        };
        self.cycle_counter += 1;

        if self.sequence_mode == SequenceMode::FourStep && self.cycle_counter == 14914 {
            self.cycle_counter = 0;
            self.irq_flag = true;
        } else if self.sequence_mode == SequenceMode::FiveStep && self.cycle_counter == 18640 {
            self.cycle_counter = 0;
            self.irq_flag = false;
        } else {
            self.irq_flag = false;
        }
        1
    }
}

impl Writer for APU {
    fn write(&mut self, address: u16, byte: u8) {
        let register = self.addr_to_register(address);
        *register = byte;
    }
}

impl Reader for APU {
    fn read(&mut self, address: u16) -> u8 {
        let register = self.addr_to_register(address);
        *register
    }
}
