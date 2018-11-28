pub mod event;
pub mod nop;
pub mod palette;
pub mod sdl;

use std::collections::VecDeque;
use std::f32::consts::PI;

use emulator::apu;
use emulator::ppu;
use emulator::state::{SaveState, ScreenState};

pub trait Graphics {
    fn draw_screen(&mut self, pixel_data: &[u8]);
}

pub struct Screen {
    scanline: u32,
    dot: u32,
    screen_buffer: [u8; 256 * 240 * 3],
    backup_buffer: [u8; 256 * 240 * 3],
    double_buffering: bool,
}

impl ppu::VideoOut for Screen {
    fn emit(&mut self, c: ppu::Colour) {
        let x = self.dot;
        let y = self.scanline;

        let (r, g, b) = palette::convert_colour(c);

        self.screen_buffer[((x + y * 256) * 3) as usize] = r;
        self.screen_buffer[((x + y * 256) * 3 + 1) as usize] = g;
        self.screen_buffer[((x + y * 256) * 3 + 2) as usize] = b;

        self.dot = (self.dot + 1) % 256;
        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 240;
            if self.scanline == 0 && self.double_buffering {
                // Flip the buffer.s
                std::mem::swap(&mut self.screen_buffer, &mut self.backup_buffer);
            }
        }
    }
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            scanline: 0,
            dot: 0,
            screen_buffer: [0; 256 * 240 * 3],
            backup_buffer: [0; 256 * 240 * 3],
            double_buffering: true,
        }
    }

    pub fn do_render<F : FnOnce(&[u8]) -> ()>(&self, render: F) {
        let buffer = if self.double_buffering { &self.backup_buffer } else { &self.screen_buffer };
        render(buffer);
    }

    pub fn set_double_buffering(&mut self, on: bool) {
        self.double_buffering = on;
    }
}

impl <'de> SaveState<'de, ScreenState> for Screen {
    fn freeze(&mut self) -> ScreenState {
        ScreenState {
            scanline: self.scanline,
            dot: self.dot,
        }
    }

    fn hydrate(&mut self, state: ScreenState) {
        self.scanline = state.scanline;
        self.dot = state.dot;
    }
}

pub struct SimpleAudioOut {
    buffer: Vec<f32>,
    counter: f32,
    fir_filter: FIRFilter,
    low_pass_filter: LowPassFilter,
    high_pass_filter_1: HighPassFilter,
    high_pass_filter_2: HighPassFilter,
    enabled: bool,
}

impl SimpleAudioOut {
    const APU_CLOCK: f32 = 1_789_772.0 / 2.0;

    pub fn new(sample_rate: f32) -> SimpleAudioOut {
        SimpleAudioOut {
            buffer: Vec::new(),
            counter: 0.0,
            // This FIR filter generated from http://t-filter.engineerjs.com/
            // Using the parameters:
            // Sample rate: 894886Hz.
            //   - from: 0Hz,   to: 20kHz,    gain: 1, ripple/att: 5dB
            //   - from: 30kHz, to: 446443Hz, gain: 0, ripple/att: -90dB
            fir_filter: FIRFilter::new(
                vec![
                -0.000026911389850328808,
                -0.00003081140855425942,
                -0.00004779032648771152,
                -0.00007032356766627306,
                -0.00009945719388661462,
                -0.0001362915825695889,
                -0.0001819696874998152,
                -0.0002376304515464224,
                -0.000304409392795344,
                -0.00038335285409846925,
                -0.0004754340414693832,
                -0.0005814565487689781,
                -0.0007020383654144685,
                -0.000837560223929689,
                -0.0009880986359432722,
                -0.0011533915706213244,
                -0.0013327996339335055,
                -0.001525238803373966,
                -0.001729170027343746,
                -0.0019425653403935548,
                -0.0021628818270826543,
                -0.0023870576384088507,
                -0.0026115231349885587,
                -0.0028322167224134323,
                -0.0030446160695497635,
                -0.003243785374697576,
                -0.0034244531543884803,
                -0.0035810842478539804,
                -0.0037079740687016955,
                -0.003799368224835729,
                -0.0038495872614102314,
                -0.0038531578568699824,
                -0.003804959012169467,
                -0.0037003696268275246,
                -0.0035354266934205804,
                -0.0033069760581996175,
                -0.003012815354076389,
                -0.0026518316881042232,
                -0.0022241292041862665,
                -0.0017311382250126026,
                -0.0011756994130167975,
                -0.0005621172682641575,
                0.0001038056638396132,
                0.0008147685021143271,
                0.001562010974441583,
                0.002335385786902094,
                0.003123468952799138,
                0.003913709351734435,
                0.0046926116716311155,
                0.005445957594914879,
                0.006159059392517742,
                0.006817043461108929,
                0.007405158670513967,
                0.007909098715661374,
                0.00831533754275437,
                0.008611474073370417,
                0.008786575131287288,
                0.008831509475056808,
                0.008739259673585048,
                0.008505206807054261,
                0.008127387262416044,
                0.007606703408582852,
                0.006947082543758409,
                0.006155584307559659,
                0.005242447599070063,
                0.0042210725502951265,
                0.0031079342495448116,
                0.0019224231576322659,
                0.0006866204855758932,
                -0.0005749961400296242,
                -0.001835912162410459,
                -0.003067993578580558,
                -0.004241957926364744,
                -0.005327890056938486,
                -0.006295795781585666,
                -0.007116190992407595,
                -0.0077606932248686,
                -0.00820261427583573,
                -0.008417549914370916,
                -0.008383946232000684,
                -0.008083629953125851,
                -0.0075022965680159445,
                -0.006629930612600777,
                -0.005461158359702242,
                -0.003995513457441722,
                -0.0022376295508218834,
                -0.00019734313868850133,
                0.0021103183631383604,
                0.0046652529398613345,
                0.007442486743310659,
                0.010412487938170444,
                0.01354154851228366,
                0.016792281277884555,
                0.02012421592250281,
                0.02349443850438209,
                0.026858268064359096,
                0.030170014467297886,
                0.03338377026120789,
                0.036454235093255336,
                0.03933744516127962,
                0.04199158082248185,
                0.04437777682414004,
                0.04646075992837699,
                0.04820945005061596,
                0.0495977101310674,
                0.050604575689545905,
                0.05121487116819005,
                0.05141933320399621,
                0.05121487116819005,
                0.050604575689545905,
                0.0495977101310674,
                0.04820945005061596,
                0.04646075992837699,
                0.04437777682414004,
                0.04199158082248185,
                0.03933744516127962,
                0.036454235093255336,
                0.03338377026120789,
                0.030170014467297886,
                0.026858268064359096,
                0.02349443850438209,
                0.02012421592250281,
                0.016792281277884555,
                0.01354154851228366,
                0.010412487938170444,
                0.007442486743310659,
                0.0046652529398613345,
                0.0021103183631383604,
                -0.00019734313868850133,
                -0.0022376295508218834,
                -0.003995513457441722,
                -0.005461158359702242,
                -0.006629930612600777,
                -0.0075022965680159445,
                -0.008083629953125851,
                -0.008383946232000684,
                -0.008417549914370916,
                -0.00820261427583573,
                -0.0077606932248686,
                -0.007116190992407595,
                -0.006295795781585666,
                -0.005327890056938486,
                -0.004241957926364744,
                -0.003067993578580558,
                -0.001835912162410459,
                -0.0005749961400296242,
                0.0006866204855758932,
                0.0019224231576322659,
                0.0031079342495448116,
                0.0042210725502951265,
                0.005242447599070063,
                0.006155584307559659,
                0.006947082543758409,
                0.007606703408582852,
                0.008127387262416044,
                0.008505206807054261,
                0.008739259673585048,
                0.008831509475056808,
                0.008786575131287288,
                0.008611474073370417,
                0.00831533754275437,
                0.007909098715661374,
                0.007405158670513967,
                0.006817043461108929,
                0.006159059392517742,
                0.005445957594914879,
                0.0046926116716311155,
                0.003913709351734435,
                0.003123468952799138,
                0.002335385786902094,
                0.001562010974441583,
                0.0008147685021143271,
                0.0001038056638396132,
                -0.0005621172682641575,
                -0.0011756994130167975,
                -0.0017311382250126026,
                -0.0022241292041862665,
                -0.0026518316881042232,
                -0.003012815354076389,
                -0.0033069760581996175,
                -0.0035354266934205804,
                -0.0037003696268275246,
                -0.003804959012169467,
                -0.0038531578568699824,
                -0.0038495872614102314,
                -0.003799368224835729,
                -0.0037079740687016955,
                -0.0035810842478539804,
                -0.0034244531543884803,
                -0.003243785374697576,
                -0.0030446160695497635,
                -0.0028322167224134323,
                -0.0026115231349885587,
                -0.0023870576384088507,
                -0.0021628818270826543,
                -0.0019425653403935548,
                -0.001729170027343746,
                -0.001525238803373966,
                -0.0013327996339335055,
                -0.0011533915706213244,
                -0.0009880986359432722,
                -0.000837560223929689,
                -0.0007020383654144685,
                -0.0005814565487689781,
                -0.0004754340414693832,
                -0.00038335285409846925,
                -0.000304409392795344,
                -0.0002376304515464224,
                -0.0001819696874998152,
                -0.0001362915825695889,
                -0.00009945719388661462,
                -0.00007032356766627306,
                -0.00004779032648771152,
                -0.00003081140855425942,
                -0.000026911389850328808,
                ]),
            low_pass_filter: LowPassFilter::new(35_000.0, SimpleAudioOut::APU_CLOCK),
            high_pass_filter_1: HighPassFilter::new(440.0, sample_rate),
            high_pass_filter_2: HighPassFilter::new(90.0, sample_rate),
            enabled: true,
        }
    }

    pub fn consume<F : FnOnce(&[f32]) -> ()>(&mut self, num_samples: usize, consume: F) {
        if self.buffer.len() == 0 || num_samples == 0 || !self.enabled {
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

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
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
