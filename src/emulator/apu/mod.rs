pub mod debug;
mod synth;

use std::cell::RefCell;
use std::rc::Rc;

use crate::emulator::clock::Ticker;
use crate::emulator::memory::{Reader, Writer};

use self::synth::{DMC, Noise, Pulse, Sweep, Triangle};

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
    irq_enabled: bool,

    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: DMC,
}

impl APU {
    pub fn new(output: Box<dyn AudioOut>, prg_rom: Box<dyn Reader>) -> APU {
        APU {
            output,

            sequence_mode: SequenceMode::FourStep,
            cycle_counter: 0,
            irq_flag: false,
            irq_enabled: true,

            pulse_1: Pulse::new(Sweep::new(false)),
            pulse_2: Pulse::new(Sweep::new(true)),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: DMC::new(prg_rom),
        }
    }

    pub fn irq_triggered(&mut self) -> bool {
        self.irq_flag || self.dmc.irq_flag
    }

    fn clock_linear_and_envelope(&mut self) {
        self.pulse_1.envelope.clock();
        self.pulse_2.envelope.clock();
        self.triangle.clock_linear();
        self.noise.envelope.clock();
    }

    fn clock_length_counters(&mut self) {
        self.pulse_1.clock_length();
        self.pulse_2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
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
                    self.irq_flag = self.irq_enabled;
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
        self.noise.clock();
        // Triangle and DMC clock twice as fast as the other components.
        self.triangle.clock();
        self.triangle.clock();
        self.dmc.clock();
        self.dmc.clock();

        // Mixer.
        let p1 = self.pulse_1.volume() as f32;
        let p2 = self.pulse_2.volume() as f32;
        let t = self.triangle.volume() as f32;
        let n = self.noise.volume() as f32;
        let dmc = self.dmc.volume as f32;

        let pulse_out = 0.00752 * (p1 + p2);
        let tnd_out = (0.00851 * t) + (0.00494 * n) + (0.00335 * dmc);
        self.output.emit(pulse_out + tnd_out);
        1
    }
}

impl Writer for APU {
    fn write(&mut self, address: u16, byte: u8) {
        match address {
            0x4000 => {
                write_first_pulse_register(&mut self.pulse_1, byte);
            },
            0x4001 => {
                write_sweep_register(&mut self.pulse_1.sweep, byte);
            }
            0x4002 => {
                write_second_pulse_register(&mut self.pulse_1, byte);
            },
            0x4003 => {
                write_third_pulse_register(&mut self.pulse_1, byte);
            },
            0x4004 => {
                write_first_pulse_register(&mut self.pulse_2, byte);
            },
            0x4005 => {
                write_sweep_register(&mut self.pulse_2.sweep, byte);
            }
            0x4006 => {
                write_second_pulse_register(&mut self.pulse_2, byte);
            },
            0x4007 => {
                write_third_pulse_register(&mut self.pulse_2, byte);
            },
            0x4008 => {
                self.triangle.linear_reload_value = byte & 0x7F;
                self.triangle.halt_length = (byte & 0x80) != 0;
                self.triangle.control_flag = (byte & 0x80) != 0;
            },
            0x400A => {
                let new_period = (self.triangle.timer.period() & 0xFF00) | (byte as u16);
                self.triangle.timer.set_period(new_period);
            },
            0x400B => {
                self.triangle.length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
                let new_period = (self.triangle.timer.period() & 0x00FF) | (((byte & 0x7) as u16) << 8);
                self.triangle.timer.set_period(new_period);
                self.triangle.linear_reload_flag = true;
            },
            0x400C => {
                self.noise.halt_length = (byte & 0x20) != 0;
                self.noise.envelope.loop_flag = (byte & 0x20) != 0;
                self.noise.envelope.constant_volume = (byte & 0x10) != 0;
                self.noise.envelope.set_volume(byte & 0x0F);
                self.noise.envelope.restart();
            },
            0x400E => {
                self.noise.mode = byte & 0x80 != 0;
                self.noise.timer.set_period(Noise::PERIOD_LOOKUP[(byte & 0x0F) as usize]);
            }
            0x400F => {
                self.noise.length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
                self.noise.envelope.restart();
            },
            0x4010 => {
                self.dmc.irq_enabled = byte & 0x80 != 0;
                self.dmc.loop_flag = byte & 0x40 != 0;
                self.dmc.timer.set_period(DMC::PERIOD_LOOKUP[(byte & 0x0F) as usize]);
            },
            0x4011 => {
                self.dmc.volume = byte & 0x7F;
            },
            0x4012 => {
                self.dmc.sample_addr = 0xC000 | ((byte as u16) << 6);
            },
            0x4013 => {
                self.dmc.sample_len = ((byte as u16) << 4) + 1;
            },
            0x4015 => {
                self.dmc.irq_flag = false;
                if (byte >> 4) & 0x1 != 0 {
                    self.dmc.enabled = true;
                    if self.dmc.bytes_remaining == 0 {
                        self.dmc.restart_sample();
                    }
                } else {
                    self.dmc.enabled = false;
                    self.dmc.bytes_remaining = 0;
                }
                if (byte >> 3) & 0x1 != 0 {
                    self.noise.enabled = true;
                } else {
                    self.noise.enabled = false;
                    self.noise.length = 0;
                }
                if (byte >> 2) & 0x1 != 0 {
                    self.triangle.enabled = true;
                } else {
                    self.triangle.enabled = false;
                    self.triangle.length = 0;
                }
                if (byte >> 1) & 0x1 != 0 {
                    self.pulse_2.enabled = true;
                } else {
                    self.pulse_2.enabled = false;
                    self.pulse_2.length = 0;
                }
                if byte & 0x1 != 0 {
                    self.pulse_1.enabled = true;
                } else {
                    self.pulse_1.enabled = false;
                    self.pulse_1.length = 0;
                }
            },
            0x4017 => {
                self.sequence_mode = if byte & 0x80 == 0 {
                    SequenceMode::FourStep
                } else {
                    SequenceMode::FiveStep
                };

                // IRQ inhibit.
                if byte & 0x70 != 0 {
                    self.irq_enabled = false;
                    self.irq_flag = false;
                } else {
                    self.irq_enabled = true;
                }

                self.cycle_counter = 0;
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
                if self.triangle.length != 0 { status |= 1 << 2 };
                if self.noise.length != 0 { status |= 1 << 3 };
                if self.dmc.bytes_remaining != 0 { status |= 1 << 4 };
                if self.dmc.irq_flag { status |= 1 << 5 };
                if self.irq_flag { status |= 1 << 6 };

                self.irq_flag = false;
                status
            },
            _ => 0,
        }
    }
}

fn write_first_pulse_register(pulse: &mut Pulse, byte: u8) {
    pulse.sequence = byte >> 6;
    // These 2 flags share the same bit.
    pulse.envelope.loop_flag = (byte & 0x20) != 0;
    pulse.halt_length = (byte & 0x20) != 0;
    pulse.envelope.constant_volume = (byte & 0x10) != 0;
    pulse.envelope.set_volume(byte & 0x0F);
    pulse.envelope.restart();
}

fn write_sweep_register(sweep: &mut Sweep, byte: u8) {
    sweep.enabled = byte & 0x80 != 0;
    sweep.divider.set_period(((byte & 0x70) >> 4) as u16);
    sweep.negate_flag = byte & 0x08 != 0;
    sweep.shift_count = byte & 0x07;
}

fn write_second_pulse_register(pulse: &mut Pulse, byte: u8) {
    let new_period = (pulse.timer.period() & 0xFF00) | (byte as u16);
    pulse.timer.set_period(new_period);
}

fn write_third_pulse_register(pulse: &mut Pulse, byte: u8) {
    pulse.length = LENGTH_COUNTER_LOOKUP[(byte >> 3) as usize];
    let new_period = (pulse.timer.period() & 0x00FF) | (((byte & 0x7) as u16) << 8);
    pulse.timer.set_period(new_period);
    pulse.restart();
}
