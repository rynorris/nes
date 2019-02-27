use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;

use emulator::components::portal::Portal;
use emulator::io::{SimpleAudioOut, Screen};
use emulator::io::event::{Event, EventHandler, Key};
use emulator::{NES, NES_MASTER_CLOCK_HZ};
use emulator::state::{load_state, save_state};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DebugMode {
    OFF,
    PPU,
    APU,
}

#[derive(Clone, Copy, Debug)]
pub struct EmulatorState {
    pub is_running: bool,
    pub is_tracing: bool,
    pub target_hz: u64,
    pub debug_mode: DebugMode,
}

impl EmulatorState {
    pub fn new() -> EmulatorState {
        EmulatorState {
            is_running: true,
            is_tracing: false,
            target_hz: NES_MASTER_CLOCK_HZ,
            debug_mode: DebugMode::APU,
        }
    }
}

pub struct Controller {
    nes: NES,
    rom_name: Option<String>,
    screen: Rc<RefCell<Screen>>,
    audio_output: Rc<RefCell<SimpleAudioOut>>,
    key_states: HashMap<Key, bool>,
    state_portal: Portal<EmulatorState>,
}

impl Controller {
    pub fn new(nes: NES,
               screen: Rc<RefCell<Screen>>,
               audio_output: Rc<RefCell<SimpleAudioOut>>,
               state_portal: Portal<EmulatorState>) -> Controller {
        Controller {
            nes,
            rom_name: None,
            screen,
            audio_output,
            key_states: HashMap::new(),
            state_portal,
        }
    }

    pub fn tick(&mut self) -> u64 {
        self.nes.tick()
    }

    pub fn is_running(&self) -> bool {
        self.state_portal.consume(|state| state.is_running)
    }

    pub fn is_tracing(&self) -> bool {
        self.state_portal.consume(|state| state.is_tracing)
    }

    pub fn set_tracing(&self, on: bool) {
        self.state_portal.consume(|state| state.is_tracing = on)
    }

    pub fn set_rom_name(&mut self, name: &str) {
        self.rom_name = Some(String::from(name));
    }

    pub fn start(&mut self) {
        self.state_portal.consume(|state| {
            state.is_running = true;
            state.is_tracing = true;
        });
        self.nes.cpu.borrow_mut().start_tracing();
    }

    pub fn stop(&mut self) {
        self.state_portal.consume(|state| {
            state.is_running = false;
        });
    }

    pub fn reset(&mut self) {
        self.nes.reset();
    }

    pub fn set_target_hz(&mut self, hz: u64) {
        self.state_portal.consume(|state| state.target_hz = hz);
        self.screen.borrow_mut().set_double_buffering(hz > 200_000);
        self.audio_output.borrow_mut().set_enabled(hz >= 10_000_000 && hz <= 50_000_000);
    }

    pub fn target_hz(&self) -> u64 {
        self.state_portal.consume(|state| state.target_hz)
    }

    pub fn debug_mode(&self) -> DebugMode {
        self.state_portal.consume(|state| state.debug_mode)
    }

    pub fn cycle_debug_mode(&self) {
        self.state_portal.consume(|state| {
            state.debug_mode = match state.debug_mode {
                DebugMode::OFF => DebugMode::PPU,
                DebugMode::PPU => DebugMode::APU,
                DebugMode::APU => DebugMode::OFF,
            };
        });
    }

    pub fn dump_trace(&mut self) {
        if self.is_tracing() {
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
        let rom_name = match self.rom_name {
            Some(ref name) => name.clone(),
            None => String::from("unknown"),
        };
        let state_name = format!("{}.{}", rom_name, num);

        if shift_modifier {
            // Save state.
            println!("Saving state: {}", state_name);
            match save_state(&mut self.nes, &state_name) {
                Err(cause) => println!("Failed to save state: {}", cause),
                Ok(_) => (),
            };
        } else if ctrl_modifier {
            // Load state.
            println!("Loading state: {}", state_name);
            match load_state(&mut self.nes, &state_name) {
                Err(cause) => println!("Failed to save state: {}", cause),
                Ok(_) => (),
            };
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
                    Key::Escape => self.stop(),
                    Key::Tab => {
                        if self.is_tracing() {
                            self.nes.cpu.borrow_mut().stop_tracing();
                            self.set_tracing(false);
                        } else {
                            self.set_tracing(true);
                            self.nes.cpu.borrow_mut().start_tracing();
                        }
                        println!("CPU Tracing: {}", if self.is_tracing() { "ON" } else { "OFF" });
                    },
                    Key::Return => {
                        self.dump_trace();
                    }
                    Key::Backquote => self.cycle_debug_mode(),
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
