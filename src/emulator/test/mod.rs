use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use emulator::ines;
use emulator::io;
use emulator::io::event::EventBus;
use emulator::io::nop::DummyGraphics;
use emulator::NES;

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

fn test_resource_path(name: &str) -> PathBuf {
    let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    buf.push("src/emulator/test/resources/");
    buf.push(name);
    buf
}
