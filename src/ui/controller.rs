use std::fs::File;

use emulator::io::event::{Event, EventHandler, Key};
use emulator::{NES, NES_MASTER_CLOCK_HZ};

pub struct Controller {
    nes: NES,
    is_running: bool,
    is_tracing: bool,
    target_hz: u64,
    show_debug: bool,
}

impl Controller {
    pub fn new(nes: NES) -> Controller {
        Controller {
            nes,
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
    }

    pub fn target_hz(&self) -> u64 {
        self.target_hz
    }

    pub fn show_debug(&self) -> bool {
        self.show_debug
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
                        println!("Flushing CPU trace buffer to ./cpu.trace");
                        let mut trace_file = match File::create("./cpu.trace") {
                            Err(_) => panic!("Couldn't open trace file"),
                            Ok(f) => f,
                        };

                        self.nes.cpu.borrow_mut().flush_trace(&mut trace_file);
                    }
                    Key::Minus => self.target_hz /= 2,
                    Key::Equals => self.target_hz *= 2,
                    Key::Num0 => self.target_hz = NES_MASTER_CLOCK_HZ,
                    Key::Num1 => self.show_debug = !self.show_debug,
                    _ => (),
                };
            },
            _ => (),
        };
    }
}
