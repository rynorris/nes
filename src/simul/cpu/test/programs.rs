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

#[test]
fn test_16bit_subtraction() {
    // Example code taken from MOS6500 programming manual.
    // 512 - 255 = 257
    // Memory: 0x0000 = 512 (little-endian), 0x0002 = 255, 0x0004 = result.
    let mut cpu = new_cpu();
    load_data(&mut cpu.memory, 0x0000, &[0b0000_0000, 0b0000_0010, 0b1111_1111, 0b0000_0000]);
    let program = [
        opcodes::SEC,
        opcodes::LDA_ZPG, 0x00,
        opcodes::SBC_ZPG, 0x02,
        opcodes::STA_ZPG, 0x04,
        opcodes::LDA_ZPG, 0x01,
        opcodes::SBC_ZPG, 0x03,
        opcodes::STA_ZPG, 0x05,
    ];
    run_program(&mut cpu, &program);
    assert_eq!(cpu.memory.load(0x0005), 0b0000_0001);
    assert_eq!(cpu.memory.load(0x0004), 0b0000_0001);
}
