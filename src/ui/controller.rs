use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;

use emulator::io::{SimpleAudioOut, SimpleVideoOut};
use emulator::io::event::{Event, EventHandler, Key};
use emulator::{NES, NES_MASTER_CLOCK_HZ};

pub struct Controller {
    nes: NES,
    video_output: Rc<RefCell<SimpleVideoOut>>,
    audio_output: Rc<RefCell<SimpleAudioOut>>,
    is_running: bool,
    is_tracing: bool,
    target_hz: u64,
    show_debug: bool,
}

impl Controller {
    pub fn new(nes: NES,
               video_output: Rc<RefCell<SimpleVideoOut>>,
               audio_output: Rc<RefCell<SimpleAudioOut>>) -> Controller {
        Controller {
            nes,
            video_output,
            audio_output,
            is_running: false,
            is_tracing: false,
            target_hz: NES_MASTER_CLOCK_HZ,
            show_debug: false,
        }
    }

    pub fn tick(&mut self) -> u64 {
        self.nes.tick()
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.is_tracing = true;
        self.nes.cpu.borrow_mut().start_tracing();
    }

    pub fn set_target_hz(&mut self, hz: u64) {
        self.target_hz = hz;
        self.video_output.borrow_mut().set_double_buffering(hz > 200_000);
        self.audio_output.borrow_mut().set_enabled(hz >= 10_000_000 && hz <= 50_000_000);
    }

    pub fn target_hz(&self) -> u64 {
        self.target_hz
    }

    pub fn show_debug(&self) -> bool {
        self.show_debug
    }

    pub fn dump_trace(&mut self) {
        if self.is_tracing {
            println!("Flushing CPU trace buffer to ./cpu.trace");
            let mut trace_file = match File::create("./cpu.trace") {
                Err(_) => panic!("Couldn't open trace file"),
                Ok(f) => f,
            };

            self.nes.cpu.borrow_mut().flush_trace(&mut trace_file);
        }
    }

    pub fn debug_print(&mut self, start: u16, len: u16) {
        println!("CPU Memory starting from ${:X}", start);
        for ix in 0 .. len {
            print!("{:02X} ", self.nes.cpu.borrow_mut().load_memory(start + ix));
        }
        println!("");
    }
}

impl EventHandler for Controller {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown(key) => {
                match key {
                    Key::Escape => self.is_running = false,
                    Key::Tab => {
                        if self.is_tracing {
                            self.nes.cpu.borrow_mut().stop_tracing();
                            self.is_tracing = false;
                        } else {
                            self.is_tracing = true;
                            self.nes.cpu.borrow_mut().start_tracing();
                        }
                        println!("CPU Tracing: {}", if self.is_tracing { "ON" } else { "OFF" });
                    },
                    Key::Return => {
                        self.dump_trace();
                    }
                    Key::Backquote => self.show_debug = !self.show_debug,
                    Key::Num1 => self.set_target_hz(0),  // Paused
                    Key::Num2 => self.set_target_hz(20_000),  // Scanlines
                    Key::Num3 => self.set_target_hz(200_000),  // Frames
                    Key::Num4 => self.set_target_hz(2_000_000),  // 1/10 slow-mo
                    Key::Num5 => self.set_target_hz(10_000_000),  // 1/2 Slow-mo
                    Key::Num6 => self.set_target_hz(NES_MASTER_CLOCK_HZ), // Normal
                    Key::Num7 => self.set_target_hz(NES_MASTER_CLOCK_HZ * 2),  // Fast Forward
                    Key::Num8 => self.set_target_hz(NES_MASTER_CLOCK_HZ * 3),
                    Key::Num9 => self.set_target_hz(NES_MASTER_CLOCK_HZ * 4),
                    Key::Num0 => self.set_target_hz(NES_MASTER_CLOCK_HZ * 5),
                    _ => (),
                };
            },
            _ => (),
        };
    }
}
