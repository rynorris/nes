use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use nes::emulator::NES;
use nes::emulator::ines;
use nes::emulator::io;
use nes::emulator::io::event::EventBus;

#[wasm_bindgen]
pub struct Emulator {
    nes: NES,
    event_bus: Rc<RefCell<EventBus>>,
    video_out: Rc<RefCell<io::Screen>>,
    audio_out: Rc<RefCell<io::SimpleAudioOut>>,
}

#[wasm_bindgen]
impl Emulator {
    pub fn new(rom: ines::ROM) -> Emulator {
            let event_bus = Rc::new(RefCell::new(EventBus::new()));
            let video_out = Rc::new(RefCell::new(io::Screen::new()));
            let audio_out = Rc::new(RefCell::new(io::SimpleAudioOut::new(48_000.0)));

            let nes = NES::new(event_bus.clone(), video_out.clone(), audio_out.clone(), rom);

            Emulator {
                nes, event_bus, video_out, audio_out,
            }
    }

    pub fn run(&mut self, ticks: u32) {
        self.nes.tick_multi(ticks);
    }

    pub fn get_frame(&self) -> Vec<u8> {
        let mut buf = [0; 256 * 240 * 3];
        self.video_out.borrow().do_render(|frame| {
            for (tgt, src) in buf.iter_mut().zip(frame.iter()) {
                *tgt = *src;
            }
        });

        return buf.to_vec();
    }
}
