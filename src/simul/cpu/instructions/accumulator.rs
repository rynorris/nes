use simul::cpu;
use simul::memory;

// LDA: Load Accumulator with Memory
// M -> A
// Affects the contents of the accumulator.
// Does not affect the carry or overflow flags.
// Sets the zero flag if the accumulator is 0 as a result, otherwise resets the zero flag.
// Sets the negative flag if bit 7 of the accumulator is 1, otherwise resets the negative flag.
pub fn lda(cpu: &mut cpu::CPU, _: &mut memory::RAM, operand_value: u8) {
    cpu.a = operand_value;

    if cpu.a == 0 {
        cpu.set_flag(cpu::Flag::Z);
    } else {
        cpu.clear_flag(cpu::Flag::Z);
    }

    if cpu.a & 0b1000_000 == 0b1000_000 {
        cpu.set_flag(cpu::Flag::N);
    } else {
        cpu.clear_flag(cpu::Flag::N);
    }
}

#[test]
fn test_lda_loads_to_accumulator() {
    let mut cpu = cpu::new();
    let mut memory = memory::new();

    lda(&mut cpu, &mut memory, 13);
    assert_eq!(cpu.a, 13);
}

#[test]
fn test_lda_sets_zero_flag() {
    let mut cpu = cpu::new();
    let mut memory = memory::new();
    assert_eq!(cpu.flag_is_set(cpu::Flag::Z), false);

    lda(&mut cpu, &mut memory, 0);
    assert_eq!(cpu.flag_is_set(cpu::Flag::Z), true);

    lda(&mut cpu, &mut memory, 1);
    assert_eq!(cpu.flag_is_set(cpu::Flag::Z), false);
}

#[test]
fn test_lda_sets_negative_flag() {
    let mut cpu = cpu::new();
    let mut memory = memory::new();
    assert_eq!(cpu.flag_is_set(cpu::Flag::N), false);

    lda(&mut cpu, &mut memory, 255);
    assert_eq!(cpu.flag_is_set(cpu::Flag::N), true);

    lda(&mut cpu, &mut memory, 1);
    assert_eq!(cpu.flag_is_set(cpu::Flag::N), false);
}
