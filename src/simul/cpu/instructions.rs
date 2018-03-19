use simul::cpu;

pub type Operation = fn(&mut cpu::CPU, cpu::addressing::AddressingMode) -> u32;

fn update_zero_flag(cpu: &mut cpu::CPU, result: u8) {
    if result == 0 {
        cpu.p.set(cpu::flags::Flag::Z);
    } else {
        cpu.p.clear(cpu::flags::Flag::Z);
    }
}

fn update_negative_flag(cpu: &mut cpu::CPU, result: u8) {
    if (result & 0b1000_0000) == 1 {
        cpu.p.set(cpu::flags::Flag::Z);
    } else {
        cpu.p.clear(cpu::flags::Flag::Z);
    }
}

// LDA: Load Accumulator with Memory
pub fn lda(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let res = cpu.load_memory(addr);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;

    addr_cycles + 1
}

// STA: Store Accumulator in Memory
pub fn sta(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.a;
    cpu.store_memory(addr, byte);
    addr_cycles + 1
}
