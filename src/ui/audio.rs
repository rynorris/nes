use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleAudioOut;

use ui::sdl2::audio;

pub struct AudioQueue {
    output: Rc<RefCell<SimpleAudioOut>>,
    queue: audio::AudioQueue<f32>,
}

impl AudioQueue {

    pub fn new(audio: sdl2::AudioSubsystem, output: Rc<RefCell<SimpleAudioOut>>) -> AudioQueue {
        let spec = audio::AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: None,
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
        self.output.borrow_mut().consume(|data| {
            let space_in_queue = (2000 - self.queue.size()) as usize;
            let samples_to_queue = if space_in_queue <= data.len() {
                space_in_queue
            } else {
                data.len()
            };
            self.queue.queue(&data[..samples_to_queue]);
        });
    }
}
