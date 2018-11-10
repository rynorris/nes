use std::cell::RefCell;
use std::rc::Rc;

use emulator::apu::APU;
use emulator::apu::synth::{Pulse};

pub struct APUDebug {
    apu: Rc<RefCell<APU>>,
}

impl APUDebug {
    pub const WAVEFORM_WIDTH: usize = 256;
    pub const WAVEFORM_HEIGHT: usize = 160;
    const WAVEFORM_SCALE: usize = 64;

    pub fn new(apu: Rc<RefCell<APU>>) -> APUDebug {
        APUDebug {
            apu,
        }
    }

    pub fn do_render<F>(&mut self, render_waveforms: F) where F: FnOnce(&[u8]) -> () {
        let mut waveform_buffer = [0; APUDebug::WAVEFORM_WIDTH * APUDebug::WAVEFORM_HEIGHT * 3];

        self.fill_waveform_buffer(&mut waveform_buffer);

        render_waveforms(&waveform_buffer);
    }

    fn fill_waveform_buffer(&self, buffer: &mut [u8]) {
        let apu = self.apu.borrow();
        APUDebug::draw_pulse_wave(buffer, &apu.pulse_1, 0, 0);
        APUDebug::draw_pulse_wave(buffer, &apu.pulse_2, 0, 32);
    }

    fn draw_pulse_wave(buffer: &mut [u8], pulse: &Pulse, x: usize, y: usize) {
        let period = pulse.timer.period();
        let amplitude = pulse.envelope.volume();
        let seq = Pulse::SEQUENCES[pulse.sequence as usize];

        if period <= 8 {
            return;
        }

        let mut prev_y = 0;
        for dx in 0 .. APUDebug::WAVEFORM_WIDTH {
            // Draw one column at a time.
            let seq_ix = (dx * APUDebug::WAVEFORM_SCALE) / (period as usize);
            let dy = (seq[seq_ix % 8] * amplitude + 8) as usize;

            if prev_y != 0 && dy != prev_y {
                // Draw vertical connecting bar.
                let (from, to) = if dy > prev_y {
                    (prev_y, dy)
                } else {
                    (dy, prev_y)
                };

                for ix in from ..= to {
                    buffer[(((y + ix) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3)] = 0xFF;
                }
            }
            prev_y = dy;

            buffer[(((y + dy) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3)] = 0xFF;
        }
    }
}
