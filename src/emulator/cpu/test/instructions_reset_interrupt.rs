use emulator::cpu::test::load_program;
use emulator::cpu::test::new_cpu;
use emulator::cpu::test::run_instructions;

#[test]
fn test_rti() {
    let mut cpu = new_cpu();
    // Push PCH, PCL, P.
    cpu.stack_push(0xBE);
    cpu.stack_push(0xEF);
    cpu.stack_push(0x34);
    load_program(&mut cpu, &[0x40]);
    let cycles = run_instructions(&mut cpu, 1);
    assert_eq!(cpu.pc, 0xBEEF);
    assert_eq!(cpu.p.as_byte(), 0x34);
    assert_eq!(cycles, 6);
}


