mod instr_misc;
mod instr_test_v5;
mod instr_timing;
mod mappers;
mod nestest;
mod ppu_sprite_hit;
mod ppu_sprite_overflow;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use self::md5::{Digest, Md5};

use crate::emulator::ines;
use crate::emulator::io;
use crate::emulator::io::event::EventBus;
use crate::emulator::io::sdl::ImageCapture;
use crate::emulator::NES;

fn run_for(nes: &mut NES, cycles: u64) {
    let mut n = 0;
    while n <= cycles {
        n += nes.tick();
    }
}

fn prepare_ete_test<P: AsRef<Path>>(path: P) -> (NES, Rc<RefCell<EventBus>>, ImageCapture) {
    let rom = ines::ROM::load(path);
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let output = Rc::new(RefCell::new(io::Screen::new()));
    let audio = io::nop::DummyAudio {};
    let image = ImageCapture::new(output.clone());
    let nes = NES::new(event_bus.clone(), output, audio, rom);
    (nes, event_bus, image)
}

fn load_and_run_blargg_test_rom<P: AsRef<Path>>(rom_path: P) -> (u8, String) {
    load_and_run_blargg_test_rom_with_cycles(rom_path, 100_000_000)
}

fn load_and_run_blargg_test_rom_with_cycles<P: AsRef<Path>>(
    rom_path: P,
    max_cycles: u64,
) -> (u8, String) {
    let (mut nes, _, _) = prepare_ete_test(rom_path);
    run_blargg_test_rom(&mut nes, max_cycles)
}

fn run_blargg_test_rom(nes: &mut NES, max_cycles: u64) -> (u8, String) {
    let mut cycles = 0;
    // Run until the status byte says the test is running.
    let mut status = nes.cpu.borrow_mut().load_memory(0x6000);
    while status != 0x80 {
        cycles += nes.tick();
        status = nes.cpu.borrow_mut().load_memory(0x6000);

        if cycles > 20_000_000 {
            panic!(
                "Test took too long to start.  Gave up after {} cycles.",
                cycles
            );
        }
    }

    // Run until completion.
    while status == 0x80 {
        cycles += nes.tick();
        status = nes.cpu.borrow_mut().load_memory(0x6000);

        cycles += 1;
        if cycles > max_cycles {
            let output = collect_test_output(nes);
            panic!(
                "Test took too long to end.  Gave up after {} cycles.  Current output: {}",
                cycles, output
            );
        }
    }

    let output = collect_test_output(nes);
    println!("{}", output);

    (status, output)
}

fn collect_test_output(nes: &mut NES) -> String {
    // Collect output.
    let mut text_buf = vec![];
    for ix in 0..1000 {
        let byte = nes.cpu.borrow_mut().load_memory(0x6004 + ix);
        if byte == 0x00 {
            break;
        } else {
            text_buf.push(byte);
        }
    }

    match String::from_utf8(text_buf) {
        Err(cause) => panic!("Error converting output to string: {}", cause),
        Ok(string) => string,
    }
}

pub fn assert_image(capture: &ImageCapture, exp_file: PathBuf) {
    let tmp_dir = env::temp_dir();
    let mut out_file = tmp_dir.clone();
    out_file.push(exp_file.file_name().unwrap());
    capture.save_bmp(&out_file);
    println!("Saving image to tempfile at: {}", out_file.display());
    assert_eq!(file_digest(out_file), file_digest(exp_file));
}

pub fn test_resource_path(name: &str) -> PathBuf {
    let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    buf.push("src/emulator/test/resources/");
    buf.push(name);
    buf
}

pub fn file_digest<P: AsRef<Path>>(path: P) -> String {
    let mut file = match File::open(&path) {
        Err(cause) => panic!("Couldn't open file: {}", cause),
        Ok(file) => file,
    };

    let mut hasher = Md5::new();

    let mut contents = vec![];
    match file.read_to_end(&mut contents) {
        Err(cause) => panic!("Couldn't read file: {}", cause),
        Ok(_) => (),
    };

    hasher.input(contents);
    base64::encode(&hasher.result())
}
