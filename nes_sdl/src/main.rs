pub mod audio;
pub mod compositor;
pub mod controller;
pub mod governer;
pub mod input;
pub mod portal;

use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use nes::emulator::apu::debug::APUDebug;
use nes::emulator::ines;
use nes::emulator::io;
use nes::emulator::io::event::{Event, EventBus};
use nes::emulator::ppu::debug::{PPUDebug, PPUDebugRender};
use nes::emulator::NES;

use crate::audio::{AudioQueue, SAMPLE_RATE};
use crate::compositor::Compositor;
use crate::controller::{Controller, DebugMode, EmulatorState};
use crate::governer::Governer;
use crate::input::InputPump;
use crate::portal::Portal;

pub const RENDER_FPS: u64 = 60;

fn main() {
    // -- Handle Args --

    let args: Vec<String> = env::args().collect();

    let rom_path = match args.get(2) {
        None => panic!("You must pass in a path to a iNes ROM file."),
        Some(path) => path,
    };

    // -- Initialize --

    let rom = ines::ROM::load(rom_path);
    let rom_name = Path::new(rom_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(String::from("unknown"));

    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let audio = sdl_context.audio().unwrap();

    let video_portal = Portal::new(vec![0; 256 * 240 * 3].into_boxed_slice());
    let ppu_debug_portal: Portal<PPUDebugRender> = Portal::new(PPUDebugRender::new());
    let apu_debug_portal = Portal::new(
        vec![0; APUDebug::WAVEFORM_WIDTH * APUDebug::WAVEFORM_HEIGHT * 3].into_boxed_slice(),
    );
    let audio_portal = Portal::new(Vec::new());
    let event_portal = Portal::new(Vec::new());

    let mut compositor = Compositor::new(
        video,
        video_portal.clone(),
        ppu_debug_portal.clone(),
        apu_debug_portal.clone(),
    );
    let mut audio_queue = AudioQueue::new(audio, audio_portal.clone());
    let mut input = InputPump::new(sdl_context.event_pump().unwrap(), event_portal.clone());

    compositor.set_window_title(&format!("[NES] {}", rom_name));

    let state = Portal::new(EmulatorState::new());
    let emu_state = state.clone();

    let ui_sync = Arc::new((Mutex::new(()), Condvar::new()));
    let emu_sync = ui_sync.clone();

    // -- Run --
    let _ = std::thread::spawn(std::panic::AssertUnwindSafe(move || {
        let event_bus = Rc::new(RefCell::new(EventBus::new()));
        let video_output = Rc::new(RefCell::new(io::Screen::new()));
        let audio_output = Rc::new(RefCell::new(io::SimpleAudioOut::new(SAMPLE_RATE)));

        let nes = NES::new(
            event_bus.clone(),
            video_output.clone(),
            audio_output.clone(),
            rom,
        );
        let ppu_debug = PPUDebug::new(nes.ppu.clone());
        let apu_debug = APUDebug::new(nes.apu.clone());

        let controller = Rc::new(RefCell::new(Controller::new(
            nes,
            video_output.clone(),
            audio_output.clone(),
            emu_state,
        )));
        controller.borrow_mut().set_rom_name(&rom_name);
        controller.borrow_mut().start();
        event_bus
            .borrow_mut()
            .register(Box::new(controller.clone()));
        main_loop(
            emu_sync,
            controller,
            video_output.clone(),
            video_portal.clone(),
            ppu_debug,
            ppu_debug_portal.clone(),
            apu_debug,
            apu_debug_portal.clone(),
            audio_output.clone(),
            audio_portal.clone(),
            event_bus.clone(),
            event_portal.clone(),
        );
    }));

    let ui_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ui_loop(
            ui_sync,
            &mut compositor,
            &mut audio_queue,
            &mut input,
            state.clone(),
        );
    }));

    match ui_res {
        Ok(_) => (),
        Err(_) => {
            println!("Panic in main loop.  Exiting.");
        }
    }
}

fn ui_loop(
    sync: Arc<(Mutex<()>, Condvar)>,
    compositor: &mut Compositor,
    audio_queue: &mut AudioQueue,
    input: &mut InputPump,
    state_portal: Portal<EmulatorState>,
) {
    while state_portal.consume(|state| state.is_running) {
        audio_queue.flush();
        compositor.render();
        input.pump();
        compositor.set_debug(state_portal.consume(|state| state.debug_mode));

        let &(ref lock, ref cvar) = &*sync;
        let guard = lock.lock().unwrap();
        let _ = cvar
            .wait_timeout(guard, Duration::from_millis(1000 / RENDER_FPS))
            .unwrap();
    }
}

fn main_loop(
    sync: Arc<(Mutex<()>, Condvar)>,
    controller: Rc<RefCell<Controller>>,
    video_output: Rc<RefCell<io::Screen>>,
    video_portal: Portal<Box<[u8]>>,
    mut ppu_debug: PPUDebug,
    ppu_debug_portal: Portal<PPUDebugRender>,
    mut apu_debug: APUDebug,
    apu_debug_portal: Portal<Box<[u8]>>,
    audio_output: Rc<RefCell<io::SimpleAudioOut>>,
    audio_portal: Portal<Vec<f32>>,
    event_bus: Rc<RefCell<EventBus>>,
    event_portal: Portal<Vec<Event>>,
) {
    let mut frame_count: u64 = 0;
    let mut agg_cycles: u64 = 0;
    let mut governer = Governer::new(RENDER_FPS);

    while controller.borrow().is_running() {
        let target_hz = controller.borrow().target_hz();
        let target_frame_cycles = target_hz / RENDER_FPS;

        let mut cycles_this_frame = 0;

        event_portal.consume(|events| {
            events
                .drain(..)
                .for_each(|e| event_bus.borrow_mut().broadcast(e));
        });

        while cycles_this_frame < target_frame_cycles && !governer.taking_too_long() {
            // Batching ticks here is a massive perf win since finding the elapsed time is costly.
            cycles_this_frame += controller.borrow_mut().tick_multi(100);
        }

        // Drive rendering.
        video_output.borrow().do_render(|data| {
            video_portal.consume(|portal| {
                copy_buffer(data, portal);
            });
        });

        match controller.borrow().debug_mode() {
            DebugMode::PPU => ppu_debug.do_render(|buffers| {
                ppu_debug_portal.consume(|portal| {
                    copy_buffer(&buffers.patterns, &mut portal.patterns);
                    copy_buffer(&buffers.nametables, &mut portal.nametables);
                    copy_buffer(&buffers.sprites, &mut portal.sprites);
                    copy_buffer(&buffers.palettes, &mut portal.palettes);
                });
            }),
            DebugMode::APU => {
                apu_debug.do_render(|data| {
                    apu_debug_portal.consume(|portal| {
                        copy_buffer(data, portal);
                    });
                });
            }
            _ => (),
        }

        let request_samples = SAMPLE_RATE / (RENDER_FPS as f32);
        audio_output
            .borrow_mut()
            .consume(target_frame_cycles, request_samples as u64, |data| {
                audio_portal.consume(|portal| {
                    portal.extend_from_slice(data);
                });
            });

        // Wake up the render thread immediately if it's waiting.
        let &(_, ref cvar) = &*sync;
        cvar.notify_one();

        governer.synchronize();

        // Calaculate stats.
        frame_count += 1;
        agg_cycles += cycles_this_frame;
        if frame_count % RENDER_FPS == 0 {
            let avg_cycles = (agg_cycles as f64) / (RENDER_FPS as f64);
            let avg_frame_ns = governer.avg_frame_duration_ns();
            let avg_hz = (avg_cycles / avg_frame_ns) * 1_000_000_000f64;
            println!(
                "FPS: {:.1}, Target Freq: {:.3}MHz, Current Freq: {:.3}MHz",
                1_000_000_000f64 / avg_frame_ns,
                (target_hz as f64) / 1_000_000f64,
                avg_hz / 1_000_000f64
            );
            agg_cycles = 0;
        }
    }
}

fn copy_buffer(src_buf: &[u8], tgt_buf: &mut [u8]) {
    for (tgt, src) in tgt_buf.iter_mut().zip(src_buf.iter()) {
        *tgt = *src;
    }
}
