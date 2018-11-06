mod synth;

use std::cell::RefCell;
use std::rc::Rc;

use emulator::clock::Ticker;
use emulator::memory::{Reader, Writer};

use self::synth::{Pulse, Triangle};

pub trait AudioOut {
    fn emit(&mut self, sample: f32);
}

impl <A : AudioOut> AudioOut for Rc<RefCell<A>> {
    fn emit(&mut self, sample: f32) {
        self.borrow_mut().emit(sample);
    }
}

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
    output: Box<AudioOut>,

    sequence_mode: SequenceMode,
    cycle_counter: u64,
    irq_flag: bool,

    pulse_1: Pulse,
    pulse_2: Pulse,

    triangle: Triangle,

    noise_timer: u8,
    noise_length: u8,
    noise_envelope: u8,
    noise_feedback: u8,
}

impl APU {
    pub fn new(output: Box<AudioOut>) -> APU {
        APU {
            output,

            sequence_mode: SequenceMode::FourStep,
            cycle_counter: 0,
            irq_flag: false,
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),

            triangle: Triangle::new(),

            noise_timer: 0,
            noise_length: 0,
            noise_envelope: 0,
            noise_feedback: 0,
        }
    }

    pub fn irq_triggered(&mut self) -> bool {
        self.irq_flag
    }

    fn clock_linear_and_envelope(&mut self) {
        self.pulse_1.envelope.clock();
        self.pulse_2.envelope.clock();
        self.triangle.clock_linear();
    }

    fn clock_length_counters(&mut self) {
        self.pulse_1.clock_length();
        self.pulse_2.clock_length();
        self.triangle.clock_linear();
    }
}

impl Ticker for APU {
    fn tick(&mut self) -> u32 {
        self.cycle_counter += 1;
        match self.sequence_mode {
            SequenceMode::FourStep => match self.cycle_counter {
                3729 => self.clock_linear_and_envelope(),
                7457 => {
                    self.clock_linear_and_envelope();
                    self.clock_length_counters();
                },
                11186 => self.clock_linear_and_envelope(),
                14915 => {
                    self.clock_linear_and_envelope();
                    self.clock_length_counters();
                    self.cycle_counter = 0;
                    self.irq_flag = true;
                },
                _ => (),
            },
            SequenceMode::FiveStep => match self.cycle_counter {
                3729 => self.clock_linear_and_envelope(),
                7457 => {
                    self.clock_linear_and_envelope();
                    self.clock_length_counters();
                },
                11186 => self.clock_linear_and_envelope(),
                18641 => {
                    self.clock_linear_and_envelope();
                    self.clock_length_counters();
                    self.cycle_counter = 0;
                },
                _ => (),
            },
        };

        self.pulse_1.clock();
        self.pulse_2.clock();
        self.triangle.clock();

        // Mixer.
        let pulse_out = 0.00752 * ((self.pulse_1.volume() + self.pulse_2.volume()) as f32);
        let tnd_out = 0.00851 * (self.triangle.volume() as f32);// + 0.00494 * noise + 0.00335 * dmc
        self.output.emit(pulse_out + tnd_out);
        1
    }
}

impl Writer for APU {
    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x4000 => {
                self.pulse_1.sequence = byte >> 6;
                // These 2 flags share the same bit.
                self.pulse_1.envelope.loop_flag = (byte & 0x20) != 0;
                self.pulse_1.halt_length = (byte & 0x20) != 0;
                self.pulse_1.envelope.constant_volume = (byte & 0x10) != 0;
                self.pulse_1.envelope.set_volume(byte & 0x0F);
                self.pulse_1.envelope.restart();
            },
            0x4001 => {
                // TODO: Sweep.
            }
            0x4002 => {
                self.pulse_1.period &= 0xFF00;
                self.pulse_1.period |= byte as u16;
            },
            0x4003 => {
                self.pulse_1.length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
                self.pulse_1.period &= 0x00FF;
                self.pulse_1.period |= ((byte & 0x7) as u16) << 8;
                self.pulse_1.restart();
            },
            0x4004 => {
                self.pulse_2.sequence = byte >> 6;
                // These 2 flags share the same bit.
                self.pulse_2.envelope.loop_flag = (byte & 0x20) != 0;
                self.pulse_2.halt_length = (byte & 0x20) != 0;
                self.pulse_2.envelope.constant_volume = (byte & 0x10) != 0;
                self.pulse_2.envelope.set_volume(byte & 0x0F);
                self.pulse_2.envelope.restart();
            },
            0x4001 => {
                // TODO: Sweep.
            }
            0x4006 => {
                self.pulse_2.period &= 0xFF00;
                self.pulse_2.period |= byte as u16;
            },
            0x4007 => {
                self.pulse_2.length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
                self.pulse_2.period &= 0x00FF;
                self.pulse_2.period |= ((byte & 0x7) as u16) << 8;
                self.pulse_2.restart();
            },
            0x4008 => {
                self.triangle.linear_reload_value = byte & 0x7F;
                self.triangle.halt_length = (byte & 0x80) != 0;
                self.triangle.control_flag = (byte & 0x80) != 0;
            },
            0x400A => {
                self.triangle.period &= 0xFF00;
                self.triangle.period |= byte as u16;
            },
            0x400B => {
                self.triangle.length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
                self.triangle.period &= 0x00FF;
                self.triangle.period |= ((byte & 0x7) as u16) << 8;
                self.triangle.linear_reload_flag = true;
            },
            0x4017 => {
                self.sequence_mode = if byte & 0x80 == 0 {
                    SequenceMode::FourStep
                } else {
                    SequenceMode::FiveStep
                };
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
                if self.pulse_1.length != 0 { status |= 1 };
                if self.pulse_2.length != 0 { status |= 1 << 1 };
                if self.irq_flag { status |= 1 << 6 };

                self.irq_flag = false;
                status
            },
            _ => 0,
        }
    }
}
