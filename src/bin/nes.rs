extern crate nes;

use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::{Duration, Instant};

use nes::emulator::{NES, NES_MASTER_CLOCK_HZ};
use nes::emulator::components::portal::Portal;
use nes::emulator::ines;
use nes::emulator::io;
use nes::emulator::io::event::{Event, EventBus};
use nes::emulator::apu::debug::APUDebug;
use nes::emulator::ppu::debug::{PPUDebug, PPUDebugRender};

use nes::ui::RENDER_FPS;
use nes::ui::audio::{AudioQueue, SAMPLE_RATE};
use nes::ui::controller::{Controller, DebugMode, EmulatorState};
use nes::ui::compositor::Compositor;
use nes::ui::input::InputPump;

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
    let apu_debug_portal = Portal::new(vec![0; APUDebug::WAVEFORM_WIDTH * APUDebug::WAVEFORM_HEIGHT * 3].into_boxed_slice());
    let audio_portal = Portal::new(Vec::new());
    let event_portal = Portal::new(Vec::new());

    let mut compositor = Compositor::new(video, video_portal.clone(), ppu_debug_portal.clone(), apu_debug_portal.clone());
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

        let nes = NES::new(event_bus.clone(), video_output.clone(), audio_output.clone(), rom);
        let ppu_debug = PPUDebug::new(nes.ppu.clone());
        let apu_debug = APUDebug::new(nes.apu.clone());

        let controller = Rc::new(RefCell::new(Controller::new(
                    nes,
                    video_output.clone(),
                    audio_output.clone(),
                    emu_state)));
        controller.borrow_mut().set_rom_name(&rom_name);
        controller.borrow_mut().start();
        event_bus.borrow_mut().register(Box::new(controller.clone()));
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
            event_portal.clone());
    }));

    let ui_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ui_loop(ui_sync, &mut compositor, &mut audio_queue, &mut input, state.clone());
    }));


    match ui_res {
        Ok(_) => (),
        Err(_) => {
            println!("Panic in main loop.  Exiting.");
        },
    }
}

fn ui_loop(
    sync: Arc<(Mutex<()>, Condvar)>,
    compositor: &mut Compositor,
    audio_queue: &mut AudioQueue,
    input: &mut InputPump,
    state_portal: Portal<EmulatorState>) {

    while state_portal.consume(|state| state.is_running) {
        audio_queue.flush();
        compositor.render();
        input.pump();
        compositor.set_debug(state_portal.consume(|state| state.debug_mode));

        let &(ref lock, ref cvar) = &*sync;
        let guard = lock.lock().unwrap();
        cvar.wait_timeout(guard, Duration::from_millis(1000 / RENDER_FPS)).unwrap();
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

    let started_instant = Instant::now();
    let frames_per_second = RENDER_FPS;
    let mut frame_start = started_instant;
    let mut frame_ix = 0;
    let mut agg_cycles = 0;
    let mut agg_start = started_instant;
    let mut oversleep_ns = 0;
    let mut overwork_cycles = 0;

    while controller.borrow().is_running() {
        let target_hz = controller.borrow().target_hz();
        let target_frame_cycles = target_hz / frames_per_second;
        let target_frame_ns = 1_000_000_000 / frames_per_second;

        let mut cycles_this_frame = 0;
        let target_ns_this_frame = target_frame_ns.saturating_sub(oversleep_ns);
        let target_cycles_this_frame = target_frame_cycles.saturating_sub(overwork_cycles);
        let mut frame_ns = 0;

        event_portal.consume(|events| {
            events.drain(..).for_each(|e| event_bus.borrow_mut().broadcast(e));
        });

        while cycles_this_frame < target_cycles_this_frame && frame_ns < target_ns_this_frame {
            // Batching ticks here is a massive perf win since finding the elapsed time is costly.
            let batch_size = 100;
            for _ in 0 .. batch_size {
                cycles_this_frame += controller.borrow_mut().tick();
            }

            let frame_time = frame_start.elapsed();
            frame_ns = frame_time.as_secs() * 1_000_000_000 + (frame_time.subsec_nanos() as u64);
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
            },
            _ => (),
        }

        let request_samples = SAMPLE_RATE / (RENDER_FPS as f32);
        audio_output.borrow_mut().consume(target_frame_cycles, request_samples as u64, |data| {
            audio_portal.consume(|portal| {
                portal.extend_from_slice(data);
            });
        });

        // Wake up the render thread immediately if it's waiting.
        let &(_, ref cvar) = &*sync;
        cvar.notify_one();

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

fn copy_buffer(src_buf: &[u8], tgt_buf:  &mut [u8]) {
    for (tgt, src) in tgt_buf.iter_mut().zip(src_buf.iter()) {
        *tgt = *src;
    }
}
