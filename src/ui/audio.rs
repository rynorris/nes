use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleAudioOut;

use ui::RENDER_FPS;
use ui::sdl2::audio;

pub const SAMPLE_RATE: f32 = 44_100.0;

pub struct AudioQueue {
    output: Rc<RefCell<SimpleAudioOut>>,
    queue: audio::AudioQueue<f32>,
}

impl AudioQueue {

    pub fn new(audio: sdl2::AudioSubsystem, output: Rc<RefCell<SimpleAudioOut>>) -> AudioQueue {
        let spec = audio::AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(1),
            samples: Some(2048),
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
        self.output.borrow_mut().consume((SAMPLE_RATE / (RENDER_FPS as f32)) as usize, |data| {
            self.queue.queue(&data);
        });
    }
}
