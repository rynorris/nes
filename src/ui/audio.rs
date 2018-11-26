use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleAudioOut;

use ui::RENDER_FPS;
use sdl2::audio;

pub const SAMPLE_RATE: f32 = 48_000.0;

pub struct AudioQueue {
    output: Rc<RefCell<SimpleAudioOut>>,
    queue: audio::AudioQueue<f32>,
}

impl AudioQueue {

    pub fn new(audio: sdl2::AudioSubsystem, output: Rc<RefCell<SimpleAudioOut>>) -> AudioQueue {
        let spec = audio::AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(1024),
        };

        let queue = match audio.open_queue(None, &spec) {
            Err(cause) => panic!("Failed to open audio queue: {}", cause),
            Ok(q) => q,
        };

        queue.resume();

        AudioQueue {
            output,
            queue,
        }
    }

    pub fn flush(&mut self) {
        let mut output = self.output.borrow_mut();
        let queue = &mut self.queue;
        let request_samples = SAMPLE_RATE / (RENDER_FPS as f32);
        output.consume(request_samples as usize, |data| {
            queue.queue(&data);
        });
    }

    pub fn size(&self) -> u32 {
        self.queue.size()
    }
}
