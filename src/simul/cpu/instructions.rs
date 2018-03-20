use simul::cpu;
use simul::util;

pub type Operation = fn(&mut cpu::CPU, cpu::addressing::AddressingMode) -> u32;

fn update_zero_flag(cpu: &mut cpu::CPU, result: u8) {
    if result == 0 {
        cpu.p.set(cpu::flags::Flag::Z);
    } else {
        cpu.p.clear(cpu::flags::Flag::Z);
    }
}

fn update_negative_flag(cpu: &mut cpu::CPU, result: u8) {
    if (result & 0b1000_0000) != 0 {
        cpu.p.set(cpu::flags::Flag::N);
    } else {
        cpu.p.clear(cpu::flags::Flag::N);
    }
}

// LDA: Load Accumulator with Memory
// A -> M
pub fn lda(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let res = cpu.load_memory(addr);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;

    addr_cycles
}

// STA: Store Accumulator in Memory
// M -> A
pub fn sta(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.a;
    cpu.store_memory(addr, byte);
    addr_cycles
}

// ADC: Add Memory to Accumulator with Carry
// A + M + C -> A, C
pub fn adc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let carry_val: u8 = if cpu.p.is_set(cpu::flags::Flag::C) { 1 } else { 0 };
    let (res, carry) = if cpu.p.is_set(cpu::flags::Flag::D) {
        // BCD arithmetic.
        let hex_a = util::bcd_to_hex(cpu.a);
        let hex_mem = util::bcd_to_hex(mem);

        // Cannot be > 255 so don't need to check for wrapping.
        let hex_res = hex_a + hex_mem + carry_val;

        // Wrap to <99.  Max value is 199 so only need to check once.
        if hex_res <= 99 {
            (util::hex_to_bcd(hex_res), false)
        } else {
            (util::hex_to_bcd(hex_res - 100), true)
        }
    } else {
        // Normal arithmetic.
        let (res, carry1) = cpu.a.overflowing_add(mem);
        let (res, carry2) = res.overflowing_add(carry_val);
        (res, carry1 || carry2)
    };
    
    // Set carry flag.
    if carry {
        cpu.p.set(cpu::flags::Flag::C);
    } else {
        cpu.p.clear(cpu::flags::Flag::C);
    }

    // Set overflow flag.
    let a_sign = cpu.a & 0b1000_0000;
    let mem_sign = mem & 0b1000_0000;
    let res_sign = res & 0b1000_0000;
    if (a_sign == mem_sign) && (a_sign != res_sign) {
        cpu.p.set(cpu::flags::Flag::V);
    } else {
        cpu.p.clear(cpu::flags::Flag::V);
    }

    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);

    cpu.a = res;
    addr_cycles
}

// SBC: Subtract Memory from Accumulator with Borrow
// A - M - ~C -> A
// Borrow = Complement of carry
pub fn sbc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let carry_val: u8 = if cpu.p.is_set(cpu::flags::Flag::C) { 1 } else { 0 };
    let (res, carry) = if cpu.p.is_set(cpu::flags::Flag::D) {
        // BCD arithmetic.
        let hex_a = util::bcd_to_hex(cpu.a);
        let hex_mem = util::bcd_to_hex(mem);
        let borrow = 1 - carry_val;
        let hex_sub_amount = hex_mem + borrow;
        let (res, carry) = hex_a.overflowing_sub(hex_sub_amount);

        // If we wrapped then we wrapped to 255.  Fudge it so we actually wrap to 99.
        if carry {
            (util::hex_to_bcd(res - (255 - 99)), carry)
        } else {
            (util::hex_to_bcd(res), carry)
        }
    } else {
        // Normal arithmetic.
        let minus_m = !mem + carry_val;
        cpu.a.overflowing_add(minus_m)
    };
    
    // Set carry flag.
    if carry {
        cpu.p.set(cpu::flags::Flag::C);
    } else {
        cpu.p.clear(cpu::flags::Flag::C);
    }

    //  Set overflow flag.
    let a_sign = cpu.a & 0b1000_0000;
    let mem_sign = mem & 0b1000_0000;
    let res_sign = res & 0b1000_0000;
    if (a_sign != mem_sign) && (mem_sign == res_sign) {
        cpu.p.set(cpu::flags::Flag::V);
    } else {
        cpu.p.clear(cpu::flags::Flag::V);
    }

    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);

    cpu.a = res;
    addr_cycles
}
