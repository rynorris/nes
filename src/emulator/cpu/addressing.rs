use emulator::cpu;
use emulator::util;

// An addressing mode calculates the final operand address, and returns it along with any extra
// cycles it too, e.g. as the result of crossing a page boundary.
// After finding the address, the function should leave the PC pointing at the next opcode.
pub type AddressingMode = fn (cpu: &mut cpu::CPU) -> (u16, u32);

fn load_memory_from_pc(cpu: &mut cpu::CPU) -> u8 {
    let addr = cpu.pc;
    cpu.load_memory(addr)
}

// Implied: no operand.
// Due to a quirk in the nature of the processor, even when doing implied addressing, 
// the CPU will read the next byte of memory and then discard it.
pub fn implied(cpu: &mut cpu::CPU) -> (u16, u32) {
    let _ = load_memory_from_pc(cpu);
    (0, 0)
}

// Immediate: one byte literal operand.
pub fn immediate(cpu: &mut cpu::CPU) -> (u16, u32) {
    let addr = cpu.pc;
    cpu.pc += 1;
    (addr, 0)
}

// Absolute: two byte operand indicates memory address.
pub fn absolute(cpu: &mut cpu::CPU) -> (u16, u32) {
    let low_byte = load_memory_from_pc(cpu);
    cpu.pc += 1;
    let high_byte = load_memory_from_pc(cpu);
    cpu.pc += 1;
    (util::combine_bytes(high_byte, low_byte), 0)
}

// Zero page: one byte operand indicates address in page 0 of memory.
pub fn zero_page(cpu: &mut cpu::CPU) -> (u16, u32) {
    let low_byte = load_memory_from_pc(cpu);
    cpu.pc += 1;
    (low_byte as u16, 0)
}

// Relative: one byte operand indicates address relative to PC.
// Only used by branch instructions.
pub fn relative(cpu: &mut cpu::CPU) -> (u16, u32) {
    let offset: u8 = load_memory_from_pc(cpu);
    cpu.pc += 1;

    // Quirk in CPU means we unnecessarily read this memory.
    let _ = load_memory_from_pc(cpu);

    // Signed addition.
    // TODO: Find out if wrapping is the correct behaviour.
    let is_negative = (offset & 0b1000_0000) != 0;
    let (addr, _) = if is_negative {
        let tc = (!offset) + 1;
        cpu.pc.overflowing_sub(tc as u16)
    } else {
        cpu.pc.overflowing_add(offset as u16)
    };

    // One extra cycle if we crossed a page boundary.
    if (addr & 0xFF00) != (cpu.pc & 0xFF00) {
        (addr, 1)
    } else {
        (addr, 0)
    }
}

// Absolute indexed: same as absolute addressing, but adds an index register to the
// address.
fn absolute_indexed_load(cpu: &mut cpu::CPU, offset: u8) -> (u16, u32) {
    let bal = load_memory_from_pc(cpu);
    cpu.pc += 1;
    let bah = load_memory_from_pc(cpu);
    cpu.pc += 1;

    let (adl, carry) = bal.overflowing_add(offset);
    if carry {
        // Quirk in CPU means we unnecessarily read this memory.
        let _ = cpu.load_memory(util::combine_bytes(bah, adl));

        let adh = bah + 1;
        (util::combine_bytes(adh, adl), 1)
    } else {
        (util::combine_bytes(bah, adl), 0)
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
// Only supported for index X except for LDX and STX.
// If the resulting value is greated than 255, the address wraps within page 0.
fn zero_page_indexed_load(cpu: &mut cpu::CPU, offset: u8) -> (u16, u32) {
    let low_byte = load_memory_from_pc(cpu);
    cpu.pc += 1;

    // Quirk in CPU means we unnecessarily read this memory.
    let _ = cpu.load_memory(low_byte as u16);

    let adjusted = (low_byte as u16) + (offset as u16);
    (adjusted & 0x00FF, 0)
}

pub fn zero_page_indexed(cpu: &mut cpu::CPU) -> (u16, u32) {
    let offset = cpu.x;
    absolute_indexed_load(cpu, offset)
}

// Y-indexed version.  Only supported for LDX, STX.
pub fn zero_page_indexed_y(cpu: &mut cpu::CPU) -> (u16, u32) {
    let offset = cpu.y;
    absolute_indexed_load(cpu, offset)
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
    let bal = load_memory_from_pc(cpu);
    cpu.pc += 1;

    // Quirk in CPU means we unnecessarily read this memory.
    let _ = cpu.load_memory(bal as u16);

    let addr = ((bal as u16) + (cpu.x as u16)) & 0x00FF;
    let adl = cpu.load_memory(addr);
    let adh = cpu.load_memory(addr + 1);
    (util::combine_bytes(adh, adl), 0)
}

pub fn indirect_indexed(cpu: &mut cpu::CPU) -> (u16, u32) {
    let ial = load_memory_from_pc(cpu);
    cpu.pc += 1;
    let bal = cpu.load_memory(ial as u16);
    let bah = cpu.load_memory((ial as u16) + 1);

    let (adl, carry) = bal.overflowing_add(cpu.y);
    if carry {
        // Quirk in CPU means we unnecessarily read this memory.
        let _ = cpu.load_memory(util::combine_bytes(bah, adl));

        let adh = bah + 1;
        (util::combine_bytes(adh, adl), 1)
    } else {
        (util::combine_bytes(bah, adl), 0)
    }
}

pub fn indirect(cpu: &mut cpu::CPU) -> (u16, u32) {
    let ial = load_memory_from_pc(cpu);
    cpu.pc += 1;
    let iah = load_memory_from_pc(cpu);
    cpu.pc += 1;
    
    let addr = util::combine_bytes(iah, ial);
    let adl = cpu.load_memory(addr);
    let adh = cpu.load_memory(addr + 1);

    (util::combine_bytes(adh, adl), 0)
}
