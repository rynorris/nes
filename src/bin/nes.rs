extern crate mos_6500;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use mos_6500::emulator::{NES, NES_MASTER_CLOCK_HZ};
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

    let output = Rc::new(RefCell::new(io::SimpleVideoOut::new()));
    let mut io = sdl::IO::new(event_bus.clone(), output.clone());

    let nes = NES::new(event_bus.clone(), output.clone(), rom);
    let lifecycle = Rc::new(RefCell::new(Lifecycle::new(nes)));

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
            for _ in 0 .. batch_size {
                cycles_this_frame += lifecycle.borrow_mut().tick();
            }

            let frame_time = frame_start.elapsed();
            frame_ns = frame_time.as_secs() * 1_000_000_000 + (frame_time.subsec_nanos() as u64);
        }

        io.tick();

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
                (current_hz as f64) / (NES_MASTER_CLOCK_HZ as f64),
                agg_ns,
                agg_cycles,
            );

            agg_cycles = 0;
        }
    }
}

pub struct Lifecycle {
    nes: NES,
    is_running: bool,
    is_tracing: bool,
    target_hz: u64,
}

impl Lifecycle {
    pub fn new(nes: NES) -> Lifecycle {
        Lifecycle {
            nes,
            is_running: false,
            is_tracing: false,
            target_hz: NES_MASTER_CLOCK_HZ,
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
}

impl EventHandler for Lifecycle {
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
                    _ => (),
                };
            },
            _ => (),
        };
    }
}

fn debug_print(nes: &mut NES, start: u16, len: u16) {
    println!("CPU Memory starting from ${:X}", start);
    for ix in 0 .. len {
        print!("{:X} ", nes.cpu.borrow_mut().load_memory(start + ix));
    }
    println!("");
}
