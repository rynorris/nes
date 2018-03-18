use simul::cpu;
use simul::util;

// An addressing mode calculates the final operand address, and returns it along with the number of
// cycles it took.
// After finding the address, the function should leave the PC pointing at the next opcode.
pub type AddressingMode = fn (cpu: &mut cpu::CPU) -> (u16, u32);

// Implied: no operand.
// Due to a quirk in the nature of the processor, even when doing implied addressing, 
// the CPU will read the next byte of memory and then discard it.
pub fn implied(cpu: &mut cpu::CPU) -> (u16, u32) {
    let _ = cpu.memory.load(cpu.pc);
    (0, 1)
}

// Immediate: one byte literal operand.
pub fn immediate(cpu: &mut cpu::CPU) -> (u16, u32) {
    let addr = cpu.pc;
    cpu.pc += 1;
    (addr, 0)
}

// Absolute: two byte operand indicates memory address.
pub fn absolute(cpu: &mut cpu::CPU) -> (u16, u32) {
    let low_byte = cpu.memory.load(cpu.pc);
    let high_byte = cpu.memory.load(cpu.pc + 1);
    cpu.pc += 2;
    (util::combine_bytes(high_byte, low_byte), 2)
}

// Zero page: one byte operand indicates address in page 0 of memory.
pub fn zero_page(cpu: &mut cpu::CPU) -> (u16, u32) {
    let low_byte = cpu.memory.load(cpu.pc);
    cpu.pc += 1;
    (low_byte as u16, 1)
}

// Relative: one byte operand indicates address relative to PC.
// Only used by branch instructions.
pub fn relative(cpu: &mut cpu::CPU) -> (u16, u32) {
    let offset = cpu.memory.load(cpu.pc);
    let is_add = (offset & 0b1000_0000) == 0x00;
    let value = offset & 0b0111_1111;
    cpu.pc += 1;
    
    // Quirk in CPU means we unnecessarily read this memory.
    let _ = cpu.memory.load(cpu.pc);
    cpu.pc += 1;

    let bah = (cpu.pc >> 8) as u8;
    let bal = cpu.pc as u8;

    let (adl, carry) = if is_add { bal.overflowing_add(value) } else { bal.overflowing_sub(value) };
    if carry {
        // Quirk in CPU means we unnecessarily read this memory.
        let _ = cpu.memory.load(util::combine_bytes(bah, adl));

        let adh = if is_add { bah + 1 } else { bal - 1 };
        (util::combine_bytes(adh, adl), 5)
    } else {
        (util::combine_bytes(bah, adl), 4)
    }
}

// Absolute indexed: same as absolute addressing, but adds an index register to the
// address.
pub fn absolute_indexed_load(cpu: &mut cpu::CPU, offset: u8) -> (u16, u32) {
    let bal = cpu.memory.load(cpu.pc);
    let bah = cpu.memory.load(cpu.pc + 1);
    cpu.pc += 2;

    let (adl, carry) = bal.overflowing_add(offset);
    if carry {
        // Quirk in CPU means we unnecessarily read this memory.
        let _ = cpu.memory.load(util::combine_bytes(bah, adl));

        let adh = bah + 1;
        (util::combine_bytes(adh, adl), 5)
    } else {
        (util::combine_bytes(bah, adl), 4)
    }
}

pub fn absolute_indexed_x(cpu: &mut cpu::CPU) -> (u16, u32) {
    let offset = cpu.x;
    absolute_indexed_load(cpu, offset)
}

pub fn absolute_indexed_y(cpu: &mut cpu::CPU) -> (u16, u32) {
    let offset = cpu.y;
    absolute_indexed_load(cpu, offset)
}

// Zero page indexed: same as zero page, but adds an index register to the address.
// Only supported for index X.
// If the resulting value is greated than 255, the address wraps within page 0.
pub fn zero_page_indexed(cpu: &mut cpu::CPU) -> (u16, u32) {
    let low_byte = cpu.memory.load(cpu.pc);
    cpu.pc += 1;

    // Quirk in CPU means we unnecessarily read this memory.
    let _ = cpu.memory.load(low_byte as u16);

    let adjusted = (low_byte as u16) + (cpu.x as u16);
    (adjusted & 0x00FF, 2)
}

// Indirect addressing is where we look up the two byte address to read from a location in page-zero.
// i.e. pointers.
//
// Indexed Indirect is where we add index X to the one byte zero page operand to find the
// lookup address. As with Zero page indexed, the resulting zero page address wraps.
//
// Indirect Indexed is where we look up the address first from the specified location in page
// zero, and _then_ add index Y to the absolute address.
//
// Indirect absolute is where we look up the address to read from another absolute address.
// This is only used by the jump instruction.
pub fn indexed_indirect(cpu: &mut cpu::CPU) -> (u16, u32) {
    let bal = cpu.memory.load(cpu.pc);
    cpu.pc += 1;

    // Quirk in CPU means we unnecessarily read this memory.
    let _ = cpu.memory.load(bal as u16);

    let addr = ((bal as u16) + (cpu.x as u16)) & 0x00FF;
    let adl = cpu.memory.load(addr);
    let adh = cpu.memory.load(addr + 1);
    (util::combine_bytes(adh, adl), 4)
}

pub fn indirect_indexed(cpu: &mut cpu::CPU) -> (u16, u32) {
    let ial = cpu.memory.load(cpu.pc);
    cpu.pc += 1;
    let bal = cpu.memory.load(ial as u16);
    let bah = cpu.memory.load((ial as u16) + 1);

    let (adl, carry) = bal.overflowing_add(cpu.y);
    if carry {
        // Quirk in CPU means we unnecessarily read this memory.
        let _ = cpu.memory.load(util::combine_bytes(bah, adl));

        let adh = bah + 1;
        (util::combine_bytes(adh, adl), 5)
    } else {
        (util::combine_bytes(bah, adl), 4)
    }
}
