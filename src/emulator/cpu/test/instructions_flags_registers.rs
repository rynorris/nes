use emulator::cpu;

use emulator::cpu::test::load_data;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_program;

#[test]
fn test_sec() {
    let mut cpu = new_cpu();
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), false);
    let cycles = run_program(&mut cpu, &[0x38]);
    assert_eq!(cpu.p.is_set(cpu::flags::Flag::C), true);
    assert_eq!(cycles, 2);
}
