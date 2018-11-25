use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;

use emulator::io::{SimpleAudioOut, Screen};
use emulator::io::event::{Event, EventHandler, Key};
use emulator::{NES, NES_MASTER_CLOCK_HZ};
use emulator::state::{load_state, save_state};
use ui::compositor::DebugMode;

pub struct Controller {
    nes: NES,
    screen: Rc<RefCell<Screen>>,
    audio_output: Rc<RefCell<SimpleAudioOut>>,
    is_running: bool,
    is_tracing: bool,
    target_hz: u64,
    debug_mode: DebugMode,
    key_states: HashMap<Key, bool>,
}

impl Controller {
    pub fn new(nes: NES,
               screen: Rc<RefCell<Screen>>,
               audio_output: Rc<RefCell<SimpleAudioOut>>) -> Controller {
        Controller {
            nes,
            screen,
            audio_output,
            is_running: false,
            is_tracing: false,
            target_hz: NES_MASTER_CLOCK_HZ,
            debug_mode: DebugMode::OFF,
            key_states: HashMap::new(),
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

    pub fn reset(&mut self) {
        self.nes.reset();
    }

    pub fn set_target_hz(&mut self, hz: u64) {
        self.target_hz = hz;
        self.screen.borrow_mut().set_double_buffering(hz > 200_000);
        self.audio_output.borrow_mut().set_enabled(hz >= 10_000_000 && hz <= 50_000_000);
    }

    pub fn target_hz(&self) -> u64 {
        self.target_hz
    }

    pub fn debug_mode(&self) -> DebugMode {
        self.debug_mode
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

    fn handle_num_key(&mut self, num: u8) {
        let shift_modifier = *self.key_states.get(&Key::Shift).unwrap_or(&false);
        let ctrl_modifier = *self.key_states.get(&Key::Control).unwrap_or(&false);

        if shift_modifier {
            // Save state.
            let name = format!("state_{}", num);
            println!("Saving state: {}", name);
            save_state(&mut self.nes, &name);
        } else if ctrl_modifier {
            // Load state.
            let name = format!("state_{}", num);
            println!("Loading state: {}", name);
            load_state(&mut self.nes, &name);
        } else {
            // Set speed.
            let target_hz = match num {
                1 => 0,  // Paused.
                2 => 20_000,  // Scanlines.
                3 => 200_000,  // Frames.
                4 => 2_000_000,  // 1/10 Slow-mo.
                5 => 10_000_000,  // 1/2 Slow-mo.
                6 => NES_MASTER_CLOCK_HZ,
                7 => NES_MASTER_CLOCK_HZ * 2,
                8 => NES_MASTER_CLOCK_HZ * 3,
                9 => NES_MASTER_CLOCK_HZ * 4,
                0 => NES_MASTER_CLOCK_HZ * 5,
                _ => panic!("Unexpected num key: {}", num),
            };
            self.set_target_hz(target_hz);
        }
    }
}

impl EventHandler for Controller {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown(key) => {
                self.key_states.insert(key, true);
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
                    Key::Backquote => self.debug_mode = match self.debug_mode {
                        DebugMode::OFF => DebugMode::PPU,
                        DebugMode::PPU => DebugMode::APU,
                        DebugMode::APU => DebugMode::OFF,
                    },
                    Key::Num1 => self.handle_num_key(1),
                    Key::Num2 => self.handle_num_key(2),
                    Key::Num3 => self.handle_num_key(3),
                    Key::Num4 => self.handle_num_key(4),
                    Key::Num5 => self.handle_num_key(5),
                    Key::Num6 => self.handle_num_key(6),
                    Key::Num7 => self.handle_num_key(7),
                    Key::Num8 => self.handle_num_key(8),
                    Key::Num9 => self.handle_num_key(9),
                    Key::Num0 => self.handle_num_key(0),
                    Key::Backspace => self.reset(),
                    _ => (),
                };
            },
            Event::KeyUp(key) => {
                self.key_states.insert(key, false);
            },
        };
    }
}
