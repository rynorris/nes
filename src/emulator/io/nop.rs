use crate::emulator::apu::AudioOut;

pub struct DummyAudio;

impl AudioOut for DummyAudio {
    fn emit(&mut self, _sample: f32) { }
}
