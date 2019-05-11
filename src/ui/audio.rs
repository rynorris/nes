use crate::emulator::components::portal::Portal;

use sdl2::audio;

pub const SAMPLE_RATE: f32 = 48_000.0;

pub struct AudioQueue {
    output: Portal<Vec<f32>>,
    queue: audio::AudioQueue<f32>,
}

impl AudioQueue {

    pub fn new(audio: sdl2::AudioSubsystem, output: Portal<Vec<f32>>) -> AudioQueue {
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
        let queue = &mut self.queue;
        self.output.consume(|data| {
            queue.queue(&data);
            data.clear();
        });
    }

    pub fn size(&self) -> u32 {
        self.queue.size()
    }
}
