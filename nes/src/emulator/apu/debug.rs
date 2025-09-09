use std::cell::RefCell;
use std::rc::Rc;

use crate::emulator::apu::synth::{Noise, Pulse, Triangle, DMC};
use crate::emulator::apu::APU;

pub struct APUDebug {
    apu: Rc<RefCell<APU>>,
    dummy_noise: Noise,
}

impl APUDebug {
    pub const WAVEFORM_WIDTH: usize = 256;
    pub const WAVEFORM_HEIGHT: usize = 160;
    const WAVEFORM_SCALE: usize = 64;

    pub fn new(apu: Rc<RefCell<APU>>) -> APUDebug {
        // We're going to pull values out of a real Noise component to get an authentic looking
        // distribution of values.  Don't want to re-implement the PRNG logic.
        // Hack the timer so we only have to clock it once to change values.
        let mut dummy_noise = Noise::new();
        dummy_noise.timer.set_period(1);
        dummy_noise.length = 1;
        dummy_noise.enabled = true;
        dummy_noise.envelope.set_volume(1);
        dummy_noise.envelope.constant_volume = true;

        APUDebug { apu, dummy_noise }
    }

    pub fn do_render<F>(&mut self, render_waveforms: F)
    where
        F: FnOnce(&[u8]) -> (),
    {
        let mut waveform_buffer = [0; APUDebug::WAVEFORM_WIDTH * APUDebug::WAVEFORM_HEIGHT * 3];

        self.fill_waveform_buffer(&mut waveform_buffer);

        render_waveforms(&waveform_buffer);
    }

    fn fill_waveform_buffer(&mut self, buffer: &mut [u8]) {
        let apu = self.apu.borrow();
        let dummy_noise = &mut self.dummy_noise;
        APUDebug::draw_pulse_wave(buffer, &apu.pulse_1, 0, 0);
        APUDebug::draw_pulse_wave(buffer, &apu.pulse_2, 0, 32);
        APUDebug::draw_triangle_wave(buffer, &apu.triangle, 0, 64);
        APUDebug::draw_noise(buffer, &apu.noise, dummy_noise, 0, 96);
        APUDebug::draw_dmc(buffer, &apu.dmc, 0, 128);
    }

    fn draw_pulse_wave(buffer: &mut [u8], pulse: &Pulse, x: usize, y: usize) {
        let period = pulse.timer.period();
        let amplitude = pulse.envelope.volume();
        let seq = Pulse::SEQUENCES[pulse.sequence as usize];

        if period <= 8 || pulse.length == 0 {
            APUDebug::draw_silence(buffer, x, y);
            return;
        }

        let mut prev_y = 0;
        for dx in 0..APUDebug::WAVEFORM_WIDTH {
            // Draw one column at a time.
            let seq_ix = (dx * APUDebug::WAVEFORM_SCALE) / (period as usize);
            let dy = (15 - seq[seq_ix % 8] * amplitude + 8) as usize;

            if prev_y != 0 && dy != prev_y {
                // Draw vertical connecting bar.
                let (from, to) = if dy > prev_y {
                    (prev_y, dy)
                } else {
                    (dy, prev_y)
                };

                for ix in from..=to {
                    buffer[((y + ix) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
                }
            }
            prev_y = dy;

            buffer[((y + dy) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
        }
    }

    fn draw_triangle_wave(buffer: &mut [u8], triangle: &Triangle, x: usize, y: usize) {
        let period = triangle.timer.period();
        if period == 0 || triangle.length == 0 || triangle.linear == 0 || !triangle.enabled {
            APUDebug::draw_silence(buffer, x, y);
            return;
        }

        for dx in 0..APUDebug::WAVEFORM_WIDTH {
            let seq_ix = (dx * APUDebug::WAVEFORM_SCALE) / (period as usize);
            let dy = (15 - Triangle::SEQUENCE[seq_ix % 32] + 8) as usize;

            buffer[((y + dy) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
        }
    }

    fn draw_noise(buffer: &mut [u8], noise: &Noise, dummy_noise: &mut Noise, x: usize, y: usize) {
        let period = noise.timer.period();

        if period == 0 || noise.length == 0 || noise.envelope.volume() == 0 {
            APUDebug::draw_silence(buffer, x, y);
            return;
        }

        dummy_noise.mode = noise.mode;

        let mut prev_seq = 0;
        let mut prev_y = 0;
        for dx in 0..APUDebug::WAVEFORM_WIDTH {
            let seq = (dx * APUDebug::WAVEFORM_SCALE) / (period as usize);
            if seq != prev_seq {
                dummy_noise.clock();
                prev_seq = seq;
            }
            // hack back in the volume from the real noise.
            let amplitude = if dummy_noise.volume() > 0 {
                noise.envelope.volume()
            } else {
                0
            };
            let dy = (15 - amplitude + 8) as usize;
            if prev_y != 0 && dy != prev_y {
                // Draw vertical connecting bar.
                let (from, to) = if dy > prev_y {
                    (prev_y, dy)
                } else {
                    (dy, prev_y)
                };

                for ix in from..=to {
                    buffer[((y + ix) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
                }
            }
            prev_y = dy;
            buffer[((y + dy) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
        }
    }

    fn draw_dmc(buffer: &mut [u8], dmc: &DMC, x: usize, y: usize) {
        // Drawing the actual sample seems too complex for little value.
        // For now just display an indicator of how much time is left on the sample.
        let bar_width = (dmc.bytes_remaining / 8) as usize;
        let inverse_width = (APUDebug::WAVEFORM_WIDTH / 2).saturating_sub(bar_width / 2);

        for dx in 0..APUDebug::WAVEFORM_WIDTH {
            if dx < inverse_width || APUDebug::WAVEFORM_WIDTH - dx < inverse_width {
                continue;
            }

            buffer[((y + 16) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;

            let half_inverse_width = (APUDebug::WAVEFORM_WIDTH / 2).saturating_sub(bar_width / 4);
            if dx < half_inverse_width || APUDebug::WAVEFORM_WIDTH - dx < half_inverse_width {
                continue;
            }
            buffer[((y + 14) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
            buffer[((y + 18) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
        }
    }

    fn draw_silence(buffer: &mut [u8], x: usize, y: usize) {
        // Base volume level is at 15, +8 to center it in the 32px tall box.
        for dx in 0..APUDebug::WAVEFORM_WIDTH {
            buffer[((y + 15 + 8) * APUDebug::WAVEFORM_WIDTH + x + dx) * 3] = 0xFF;
        }
    }
}
