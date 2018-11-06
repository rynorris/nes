pub mod event;
pub mod nop;
pub mod palette;
pub mod sdl;

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
    low_pass_filter: LowPassFilter,
    high_pass_filter_1: HighPassFilter,
    high_pass_filter_2: HighPassFilter,
}

impl SimpleAudioOut {
    const BUF_SIZE: usize = 2048;
    const APU_CLOCK: f32 = 1_789_773.0 / 2.0;
    const SAMPLE_RATE: f32 = 44_100.0;
    const SAMPLE_CYCLES: f32 = SimpleAudioOut::APU_CLOCK / SimpleAudioOut::SAMPLE_RATE;

    pub fn new() -> SimpleAudioOut {
        SimpleAudioOut {
            buffer: Vec::with_capacity(SimpleAudioOut::BUF_SIZE),
            counter: 0.0,
            low_pass_filter: LowPassFilter::new(14_000.0, SimpleAudioOut::SAMPLE_RATE),
            high_pass_filter_1: HighPassFilter::new(440.0, SimpleAudioOut::SAMPLE_RATE),
            high_pass_filter_2: HighPassFilter::new(90.0, SimpleAudioOut::SAMPLE_RATE),
        }
    }

    pub fn consume<F : FnOnce(&[f32]) -> ()>(&mut self, consume: F) {
        consume(self.buffer.as_slice());
        self.buffer.clear();
    }

    fn queue_sample(&mut self, mut sample: f32) {
        sample = self.low_pass_filter.process(sample);
        sample = self.high_pass_filter_1.process(sample);
        sample = self.high_pass_filter_2.process(sample);
        // Just drop samples if our buffer is full.
        if self.buffer.len() <= SimpleAudioOut::BUF_SIZE {
            self.buffer.push(sample);
        }
    }
}

impl apu::AudioOut for SimpleAudioOut {
    fn emit(&mut self, sample: f32) {
        // Drop most samples.
        self.counter += 1.0;
        if self.counter > SimpleAudioOut::SAMPLE_CYCLES {
            self.counter -= SimpleAudioOut::SAMPLE_CYCLES;
            self.queue_sample(sample);
        }
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
            alpha: rc / (rc + dt),
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
