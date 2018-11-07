pub mod event;
pub mod nop;
pub mod palette;
pub mod sdl;

use std::collections::VecDeque;
use std::f32::consts::PI;

use emulator::apu;
use emulator::ppu;

pub trait Graphics {
    fn draw_screen(&mut self, pixel_data: &[u8]);
}

pub struct SimpleVideoOut {
    scanline: u32,
    dot: u32,
    screen_buffer: [u8; 256 * 240 * 3],
    render_tile_grid: bool,
}

impl ppu::VideoOut for SimpleVideoOut {
    fn emit(&mut self, c: ppu::Colour) {
        let x = self.dot;
        let y = self.scanline;

        let (r, g, b) = if self.render_tile_grid && (x % 8 == 0 || y % 8 == 0) {
            (255, 0, 0)
        } else {
            palette::convert_colour(c)
        };

        self.screen_buffer[((x + y * 256) * 3) as usize] = r;
        self.screen_buffer[((x + y * 256) * 3 + 1) as usize] = g;
        self.screen_buffer[((x + y * 256) * 3 + 2) as usize] = b;

        self.dot = (self.dot + 1) % 256;
        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 240;
        }
    }
}

impl SimpleVideoOut {
    pub fn new() -> SimpleVideoOut {
        SimpleVideoOut {
            scanline: 0,
            dot: 0,
            screen_buffer: [0; 256 * 240 * 3],
            render_tile_grid: false,
        }
    }

    pub fn do_render<F : FnOnce(&[u8]) -> ()>(&self, render: F) {
        render(&self.screen_buffer);
    }
}

pub struct SimpleAudioOut {
    buffer: Vec<f32>,
    counter: f32,
    fir_filter: FIRFilter,
    low_pass_filter: LowPassFilter,
    high_pass_filter_1: HighPassFilter,
    high_pass_filter_2: HighPassFilter,
}

impl SimpleAudioOut {
    const APU_CLOCK: f32 = 1_789_772.0 / 2.0;

    pub fn new(sample_rate: f32) -> SimpleAudioOut {
        SimpleAudioOut {
            buffer: Vec::new(),
            counter: 0.0,
            // This FIR filter generated from http://t-filter.engineerjs.com/
            // Using the parameters:
            //   - from: 0Hz,   to: 20kHz,    gain: 1, ripple/att: 5dB
            //   - from: 40kHz, to: 446443Hz, gain: 0, ripple/att: -50dB
            fir_filter: FIRFilter::new(vec![
               -0.01340311369837813,
                0.01476507963675876,
                -0.03353697526740505,
                0.04458701449082413,
                -0.07594796939730486,
                0.10084297887139906,
                -0.14794808614326035,
                0.1935159059359888,
                -0.2583565898359244,
                0.331357362915934,
                -0.4139186418164375,
                0.5197548135657892,
                -0.6172899443686382,
                0.7587263074056847,
                -0.8653362574271454,
                1.0413647050134642,
                -1.1480883817029024,
                1.3533150306146828,
                -1.448821790556084,
                1.6735321544125024,
                -1.7453277479968867,
                1.9763013415115769,
                -2.0123870774857764,
                2.2343748714588205,
                -2.2250387308801765,
                2.4226887590369417,
                -2.362125669030104,
                2.5220512596448814,
                -2.4094838495067536,
                2.5220512596448814,
                -2.362125669030104,
                2.4226887590369417,
                -2.2250387308801765,
                2.2343748714588205,
                -2.0123870774857764,
                1.9763013415115769,
                -1.7453277479968867,
                1.6735321544125024,
                -1.448821790556084,
                1.3533150306146828,
                -1.1480883817029024,
                1.0413647050134642,
                -0.8653362574271454,
                0.7587263074056847,
                -0.6172899443686382,
                0.5197548135657892,
                -0.4139186418164375,
                0.331357362915934,
                -0.2583565898359244,
                0.1935159059359888,
                -0.14794808614326035,
                0.10084297887139906,
                -0.07594796939730486,
                0.04458701449082413,
                -0.03353697526740505,
                0.01476507963675876,
                -0.01340311369837813,
            ]),
            low_pass_filter: LowPassFilter::new(35_000.0, SimpleAudioOut::APU_CLOCK),
            high_pass_filter_1: HighPassFilter::new(440.0, sample_rate),
            high_pass_filter_2: HighPassFilter::new(90.0, sample_rate),
        }
    }

    pub fn consume<F : FnOnce(&[f32]) -> ()>(&mut self, num_samples: usize, consume: F) {
        if self.buffer.len() == 0 || num_samples == 0 {
            return;
        }

        let mut buf = Vec::with_capacity(num_samples);

        // Need to downsample all the samples we collected this frame.
        let total = self.buffer.len();
        let step = (total as f32) / (num_samples as f32);
        let mut counter = 0.0;
        for ix in 0 .. total {
            self.fir_filter.shift(self.buffer[ix]);

            counter += 1.0;
            if counter >= step {
                counter -= step;
                let sample = self.fir_filter.compute();
                buf.push(sample);
            }
        }
        
        consume(&buf);
        self.buffer.clear();
    }

    fn queue_sample(&mut self, sample: f32) {
        self.buffer.push(sample);
    }
}

impl apu::AudioOut for SimpleAudioOut {
    fn emit(&mut self, sample: f32) {
        self.queue_sample(sample);
    }
}

struct LowPassFilter {
    prev_out: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(freq: f32, sample_rate: f32) -> LowPassFilter {
        let rc = 1.0 / (2.0 * PI * freq);
        let dt = 1.0 / sample_rate;
        LowPassFilter {
            prev_out: 0.0,
            alpha: dt / (rc + dt),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        // y[i] := y[i-1] + α * (x[i] - y[i-1])
        let out = self.prev_out + self.alpha * (sample - self.prev_out);
        self.prev_out = out;
        out
    }
}

struct HighPassFilter {
    prev_in: f32,
    prev_out: f32,
    alpha: f32,
}

impl HighPassFilter {
    pub fn new(freq: f32, sample_rate: f32) -> HighPassFilter {
        let rc = 1.0 / (2.0 * PI * freq);
        let dt = 1.0 / sample_rate;
        HighPassFilter {
            prev_in: 0.0,
            prev_out: 0.0,
            alpha: rc / (rc + dt),
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        // y[i] := α * (y[i-1] + x[i] - x[i-1])
        let out = self.alpha * (self.prev_out + sample - self.prev_in);
        self.prev_in = sample;
        self.prev_out = out;
        out
    }
}

struct FIRFilter {
    impulse_response: Vec<f32>,
    buffer: VecDeque<f32>,
}

impl FIRFilter {
    pub fn new(impulse_response: Vec<f32>) -> FIRFilter  {
        let order = impulse_response.len();
        let mut buffer = VecDeque::with_capacity(order);
        for _ in 0 .. order {
            buffer.push_back(0.0);
        }

        FIRFilter {
            impulse_response,
            buffer,
        }
    }

    pub fn shift(&mut self, sample: f32) {
        self.buffer.pop_front();
        self.buffer.push_back(sample);
    }

    pub fn compute(&self) -> f32 {
        self.compute_sample()
    }

    fn compute_sample(&self) -> f32 {
        let mut sample = 0.0;
        for ix in 0 .. self.impulse_response.len() {
            sample += self.buffer[ix] * self.impulse_response[ix];
        }
        sample
    }
}
