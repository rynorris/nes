extern crate base64;
extern crate md5;

use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use self::md5::{Md5, Digest};

use emulator::ines;
use emulator::io;
use emulator::io::event::{Event, EventBus, Key};
use emulator::io::nop::DummyGraphics;
use emulator::io::sdl::ImageCapture;
use emulator::NES;

// -- Visual nestest.
#[test]
fn test_nestest_visual() {
    let path = test_resource_path("nestest/nestest.nes");
    let rom = ines::ROM::load(&path.into_os_string().into_string().unwrap());
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let graphics = Rc::new(RefCell::new(ImageCapture::new()));
    let output = io::SimpleVideoOut::new(graphics.clone());
    let mut nes = NES::new(event_bus.clone(), output, rom);

    let tmp_dir = env::temp_dir();

    // Wait for main menu to load.
    for _ in 1 .. 500_000 {
        nes.tick();
    }

    // Check the menu loaded properly.
    let mut bmp_01_path = tmp_dir.clone();
    bmp_01_path.push("01.bmp");
    graphics.borrow_mut().save_bmp(&bmp_01_path);
    println!("Saving image to tempfile at: {}", bmp_01_path.display());
    assert_eq!(file_digest(bmp_01_path), file_digest(test_resource_path("nestest/capture_01_menu.bmp")));

    // Start tests.
    event_bus.borrow_mut().broadcast(Event::KeyDown(Key::A));
   
    // Wait for tests to finish.
    for _ in 1 .. 2_000_000 {
        nes.tick();
    }

    // Check the tests passed.
    let mut bmp_02_path = tmp_dir.clone();
    bmp_02_path.push("02.bmp");
    println!("Saving image to tempfile at: {}", bmp_02_path.display());
    graphics.borrow_mut().save_bmp(&bmp_02_path);
    assert_eq!(file_digest(bmp_02_path), file_digest(test_resource_path("nestest/capture_02_passed.bmp")));
}

// -- instr_misc test ROMs --
// TODO: Get 03 and 04 to pass and add tests for them.
#[test]
fn test_instr_misc_01() {
    let path = test_resource_path("instr_misc/rom_singles/01-abs_x_wrap.nes");
    let (status, output) = load_and_run_blargg_test_rom(&path.into_os_string().into_string().unwrap());

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n01-abs_x_wrap\n\nPassed\n");
}

#[test]
fn test_instr_misc_02() {
    let path = test_resource_path("instr_misc/rom_singles/02-branch_wrap.nes");
    let (status, output) = load_and_run_blargg_test_rom(&path.into_os_string().into_string().unwrap());

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n02-branch_wrap\n\nPassed\n");
}

fn test_instr_misc_03() {
    let path = test_resource_path("instr_misc/rom_singles/03-dummy_reads.nes");
    let (status, output) = load_and_run_blargg_test_rom(&path.into_os_string().into_string().unwrap());

    assert_eq!(status, 0x00);
    assert_eq!(output, "\n02-branch_wrap\n\nPassed\n");
}

fn load_and_run_blargg_test_rom(rom_path: &str) -> (u8, String) {
    let rom = ines::ROM::load(rom_path);
    let event_bus = Rc::new(RefCell::new(EventBus::new()));
    let graphics = Rc::new(RefCell::new(DummyGraphics{}));
    let output = io::SimpleVideoOut::new(graphics.clone());
    let mut nes = NES::new(event_bus.clone(), output, rom);

    run_blargg_test_rom(&mut nes)
}

fn run_blargg_test_rom(nes: &mut NES) -> (u8, String) {
    let mut cycles = 0;
    // Run until the status byte says the test is running.
    let mut status = nes.cpu.borrow_mut().load_memory(0x6000);
    while status != 0x80 {
        cycles += nes.tick();
        status = nes.cpu.borrow_mut().load_memory(0x6000);

        if cycles > 20_000_000 {
            panic!("Test took too long to start.  Gave up after {} cycles.", cycles);
        }
    }

    // Run until completion.
    while status == 0x80 {
        cycles += nes.tick();
        status = nes.cpu.borrow_mut().load_memory(0x6000);

        cycles += 1;
        if cycles > 50_000_000 {
            let output = collect_test_output(nes);
            panic!("Test took too long to end.  Gave up after {} cycles.  Current output: {}", cycles, output);
        }
    }

    let output = collect_test_output(nes);
    println!("{}", output);

    (status, output)
}

fn collect_test_output(nes: &mut NES) -> String {
    // Collect output.
    let mut text_buf = vec![];
    for ix in 0 .. 1000 {
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
