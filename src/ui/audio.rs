use std::io::{BufWriter, Write};
use std::fs::File;
use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::SimpleAudioOut;

use ui::RENDER_FPS;
use ui::sdl2::audio;

pub const SAMPLE_RATE: f32 = 48_000.0;

pub struct AudioQueue {
    output: Rc<RefCell<SimpleAudioOut>>,
    queue: audio::AudioQueue<f32>,
    file: BufWriter<File>,
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

        let file = match File::create("audio.wav") {
            Err(cause) => panic!("Failed to open audio file: {}", cause),
            Ok(f) => f,
        };

        queue.resume();

        AudioQueue {
            output,
            queue,
            file: BufWriter::new(file),
        }
    }

    pub fn flush(&mut self) {
        let mut output = self.output.borrow_mut();
        let queue = &mut self.queue;
        let file = &mut self.file;
        let request_samples = SAMPLE_RATE / (RENDER_FPS as f32);
        output.consume(request_samples as usize, |data| {
            queue.queue(&data);
            let mut frame_file = match File::create("audio_frame.wav") {
                Err(cause) => panic!("Failed to open audio file: {}", cause),
                Ok(f) => f,
            };
            for ix in 0 .. data.len() {
                let pcm_sample = (data[ix] * 128.0 + 128.0) as u8;
                file.write(&[pcm_sample]).unwrap();
                frame_file.write(&[pcm_sample]).unwrap();
            }
        });
    }

    pub fn size(&self) -> u32 {
        self.queue.size()
    }
}
