extern crate mos_6500;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use mos_6500::emulator;
use mos_6500::emulator::ines;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_path = match args.get(2) {
        None => panic!("You must pass in a path to a iNes ROM file."),
        Some(path) => path,
    };

    let seconds_to_run: u64 = match args.get(3) {
        None => 10,
        Some(val) => match val.parse() {
            Err(cause) => panic!("Failed to parse seconds_to_run: {}", cause),
            Ok(num) => num,
        },
    };

    let trace_enabled = match env::var("NES_TRACE") {
        Err(_) => false,
        Ok(val) => val == "1",
    };

    let trace_out_path = Path::new("./cpu.trace");
    let trace_out = match File::create(trace_out_path) {
        Err(cause) => panic!("Couldn't open {}: {}", trace_out_path.display(), cause),
        Ok(file) => file,
    };

    let rom = ines::ROM::load(rom_path);

    let mut nes = emulator::NES::new(rom);

    while nes.elapsed_seconds() < seconds_to_run {
        for _ in 0 .. 10000 {
            if trace_enabled {
                nes.cpu.borrow_mut().trace_next_instruction(&trace_out);
                write!(&trace_out, "\n");
            }
            nes.tick();
        }
    }
}
