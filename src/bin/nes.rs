extern crate mos_6500;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use mos_6500::emulator;
use mos_6500::emulator::clock::Ticker;
use mos_6500::emulator::ines;
use mos_6500::emulator::io;
use mos_6500::emulator::io::event::{Event, EventBus, EventHandler, Key};
use mos_6500::emulator::io::sdl;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_path = match args.get(2) {
        None => panic!("You must pass in a path to a iNes ROM file."),
        Some(path) => path,
    };

    let rom = ines::ROM::load(rom_path);

    let event_bus = Rc::new(RefCell::new(EventBus::new()));

    let io = Rc::new(RefCell::new(sdl::IO::new(event_bus.clone())));
    let output = io::SimpleVideoOut::new(io.clone());

    let mut nes = emulator::NES::new(event_bus.clone(), output, rom);

    let lifecycle = Rc::new(RefCell::new(Lifecycle::new()));
    lifecycle.borrow_mut().start();
    event_bus.borrow_mut().register(Box::new(lifecycle.clone()));

    let started_instant = Instant::now();
    let frames_per_second = 30;
    let mut frame_start = started_instant;
    let mut frame_ix = 0;
    let mut agg_cycles = 0;
    let mut agg_start = started_instant;
    let mut oversleep_ns = 0;
    let mut overwork_cycles = 0;

    while lifecycle.borrow().is_running() {
        let target_hz = lifecycle.borrow().target_hz();
        let target_frame_cycles = target_hz / frames_per_second;
        let target_frame_ns = 1_000_000_000 / frames_per_second;

        let mut cycles_this_frame = 0;
        let target_ns_this_frame = target_frame_ns.saturating_sub(oversleep_ns);
        let target_cycles_this_frame = target_frame_cycles - overwork_cycles;
        let mut frame_ns = 0;

        while cycles_this_frame < target_cycles_this_frame && frame_ns < target_ns_this_frame {
            // Batching ticks here is a massive perf win since finding the elapsed time is costly.
            let batch_size = 100;
            for _ in 1 .. batch_size {
                lifecycle.borrow_mut().trace_next_instruction(&nes);
                cycles_this_frame += nes.tick();
            }

            let frame_time = frame_start.elapsed();
            frame_ns = frame_time.as_secs() * 1_000_000_000 + (frame_time.subsec_nanos() as u64);
        }

        io.borrow_mut().tick();

        // If we finished early then calculate sleep and stuff, otherwise just plough onwards.
        if frame_ns < target_ns_this_frame {
            let render_end = Instant::now();
            let render_time = render_end - frame_start;
            let render_ns = render_time.as_secs() * 1_000_000_000 + (render_time.subsec_nanos() as u64);
            let sleep_ns = target_ns_this_frame.saturating_sub(render_ns);

            thread::sleep(Duration::from_nanos(sleep_ns));
        }

        let frame_end = Instant::now();
        // If we slept too long, take that time off the next frame.
        oversleep_ns = ((frame_end - frame_start).subsec_nanos() as u64).saturating_sub(target_ns_this_frame);
        overwork_cycles = cycles_this_frame.saturating_sub(target_cycles_this_frame);
        frame_start = frame_end;
        
        // Print debug info here.
        agg_cycles += cycles_this_frame;
        frame_ix = (frame_ix + 1) % frames_per_second;
        if frame_ix == 0 {
            let agg_duration = agg_start.elapsed();
            agg_start = Instant::now();

            let agg_ns = agg_duration.as_secs() * 1_000_000_000 + (agg_duration.subsec_nanos() as u64);
            let current_hz = (agg_cycles * 1_000_000_000) / agg_ns;

            println!(
                "Target: {:.3}MHz, Current: {:.3}MHz ({:.2}x).  Took: {}ns to process {} cycles.",
                (target_hz as f64) / 1_000_000f64,
                (current_hz as f64) / 1_000_000f64,
                (current_hz as f64) / (emulator::NES_MASTER_CLOCK_HZ as f64),
                agg_ns,
                agg_cycles,
            );

            agg_cycles = 0;
        }
    }
}

pub struct Lifecycle {
    is_running: bool,
    trace_file: Option<File>,
    target_hz: u64,
}

impl Lifecycle {
    pub fn new() -> Lifecycle {
        Lifecycle {
            is_running: false,
            trace_file: None,
            target_hz: emulator::NES_MASTER_CLOCK_HZ,
        }
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

    pub fn trace_next_instruction(&mut self, nes: &emulator::NES) {
        if let Some(f) = self.trace_file.as_mut() {
            nes.cpu.borrow_mut().trace_next_instruction(&*f);
            write!(f, "\n");
        }
    }
}

impl EventHandler for Lifecycle {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown(key) => {
                match key {
                    Key::Escape => self.is_running = false,
                    Key::Tab => {
                        if self.trace_file.is_some() {
                            return;
                        }
                        let trace_file = match File::create("./cpu.trace") {
                            Err(_) => panic!("Couldn't open trace file"),
                            Ok(f) => f,
                        };
                        self.trace_file = Some(trace_file);
                    },
                    Key::Minus => self.target_hz /= 2,
                    Key::Equals => self.target_hz *= 2,
                    Key::Num0 => self.target_hz = emulator::NES_MASTER_CLOCK_HZ,
                    _ => (),
                };
            },
            Event::KeyUp(key) => {
                match key {
                    Key::Tab => {
                        self.trace_file = None;
                    },
                    _ => (),
                };
            },
        };
    }
}

fn debug_print(nes: &mut emulator::NES, start: u16, len: u16) {
    println!("CPU Memory starting from ${:X}", start);
    for ix in 0 .. len {
        print!("{:X} ", nes.cpu.borrow_mut().load_memory(start + ix));
    }
    println!("");
}
