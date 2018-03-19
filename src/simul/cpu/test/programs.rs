use simul::cpu::opcodes;

use simul::cpu::test::load_data;
use simul::cpu::test::new_cpu;
use simul::cpu::test::run_program;

#[test]
fn test_load_add_save() {
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0099, &[0x34]);
    run_program(&mut cpu, &[
                opcodes::LDA_ZPG, 0x99,  // LDA 0x34
                opcodes::ADC_IMM, 0x56,  // ADC # 0x56
                opcodes::STA_ABS, 0xEF, 0xBE  // STA 0xBEEF
    ]);
    assert_eq!(cpu.memory.load(0xBEEF), 0x8A);
}
