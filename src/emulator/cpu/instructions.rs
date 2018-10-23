use emulator::cpu;
use emulator::util;

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

fn load_status_from_stack(cpu: &mut cpu::CPU) {
    let bits_from_stack = cpu.stack_pop() & 0b1100_1111;
    let bits_from_register = cpu.p.as_byte() & 0b0011_0000;
    cpu.p.load_byte(bits_from_stack | bits_from_register);
}

/* 2.1 The Accumulator */

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
    // STA doesn't incur the extra "oops" cycle.
    // Or more correctly, it always does, but it's taken into account by the
    // instruction timings already, so we ignore it here.
    0
}

/* 2.2 The Arithmetic Unit */

// ADC: Add Memory to Accumulator with Carry
// A + M + C -> A, C
pub fn adc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let carry_val: u8 = if cpu.p.is_set(cpu::flags::Flag::C) { 1 } else { 0 };
    let (res, carry) = if cpu.p.is_set(cpu::flags::Flag::D) && cpu.dec_arith_on {
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
    let (res, carry) = if cpu.p.is_set(cpu::flags::Flag::D) && cpu.dec_arith_on {
        // BCD arithmetic.
        let hex_a = util::bcd_to_hex(cpu.a);
        let hex_mem = util::bcd_to_hex(mem);
        let borrow = 1 - carry_val;
        let hex_sub_amount = hex_mem + borrow;
        let (res, borrow) = hex_a.overflowing_sub(hex_sub_amount);

        // If we wrapped then we wrapped to 255.  Fudge it so we actually wrap to 99.
        if borrow {
            (util::hex_to_bcd(res - (255 - 99)), false)
        } else {
            (util::hex_to_bcd(res), true)
        }
    } else {
        // Normal arithmetic.
        let (minus_m, _) = (!mem).overflowing_add(carry_val);
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

// AND: Bitwise AND Memory with Accumulator
// A /\ M -> A
pub fn and(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);
    let res = mem & cpu.a;
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;
    addr_cycles
}

// ORA: Bitwise OR Memory with Accumulator
// A \/ M -> A
pub fn ora(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);
    let res = mem | cpu.a;
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;
    addr_cycles
}

// EOR: Bitwise Exclusive OR Memory with Accumulator
// A \-/ M -> A
pub fn eor(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);
    let res = mem ^ cpu.a;
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.a = res;
    addr_cycles
}

/* 3. Flags and Status Register */

// SEC: Set Carry Flag
// 1 -> C
pub fn sec(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.set(cpu::flags::Flag::C);
    0
}

// CLC: Clear Carry Flag
// 0 -> C
pub fn clc(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::C);
    0
}

// SEI: Set Interrupt Disable
// 1 -> I
pub fn sei(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.set(cpu::flags::Flag::I);
    0
}

// CLI: Clear Interrupt Disable
// 0 -> I
pub fn cli(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::I);
    0
}

// SED: Set Decimal Mode
// 1 -> D
pub fn sed(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.set(cpu::flags::Flag::D);
    0
}

// CLD: Clear Decimal Mode
// 0 -> D
pub fn cld(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::D);
    0
}

// CLV: Clear Overflow Flag
// 0 -> V
pub fn clv(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.p.clear(cpu::flags::Flag::V);
    0
}

/* 4. Test, Branch and Jump Instructions */

// JMP: Jump to New Location
// (PC + 1) -> PCL, (PC + 2) -> PCH
pub fn jmp(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    cpu.pc = addr;
    addr_cycles
}

// Common functionality for branch instructions.
fn branch_if(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode, should_branch: bool) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    if should_branch {
        cpu.pc = addr;
        addr_cycles + 1
    } else {
        // If not branching we don't incur any of the extra cycles.
        0
    }
}

// BMI - Branch on Result Minus
pub fn bmi(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::N);
    branch_if(cpu, load_addr, should_branch)
}

// BPL - Branch on Result Plus
pub fn bpl(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::N);
    branch_if(cpu, load_addr, should_branch)
}

// BCC - Branch on Carry Clear
pub fn bcc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::C);
    branch_if(cpu, load_addr, should_branch)
}

// BCS - Branch on Carry Set
pub fn bcs(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::C);
    branch_if(cpu, load_addr, should_branch)
}

// BEQ - Branch on Result Zero
pub fn beq(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::Z);
    branch_if(cpu, load_addr, should_branch)
}

// BNE - Branch on Result Not Zero
pub fn bne(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::Z);
    branch_if(cpu, load_addr, should_branch)
}

// BVS - Branch on Overflow Set
pub fn bvs(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = cpu.p.is_set(cpu::flags::Flag::V);
    branch_if(cpu, load_addr, should_branch)
}

// BVC - Branch on Overflow Clear
pub fn bvc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let should_branch = !cpu.p.is_set(cpu::flags::Flag::V);
    branch_if(cpu, load_addr, should_branch)
}

fn compare_instruction(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode, compare_with: u8) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    let diff = compare_with.wrapping_sub(mem);
    update_zero_flag(cpu, diff);
    update_negative_flag(cpu, diff);

    if compare_with < mem {
        cpu.p.clear(cpu::flags::Flag::C);
    } else {
        cpu.p.set(cpu::flags::Flag::C);
    }

    addr_cycles
}

// CMP - Compare Memory and Accumulator
// A - M
pub fn cmp(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    compare_instruction(cpu, load_addr, byte)
}

// BIT: Test Bits in Memory with Accumulator
// M /\ A, M7 -> N, M6 -> V
pub fn bit(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    // N is set to bit 7 of the memory being tested.
    update_negative_flag(cpu, mem);

    // V is set to bit 6 of the memory being tested.
    if (mem & 0b0100_0000) != 0 {
        cpu.p.set(cpu::flags::Flag::V);
    } else {
        cpu.p.clear(cpu::flags::Flag::V);
    }

    // Z is set if M AND A is 0.
    if (mem & cpu.a) == 0 {
        cpu.p.set(cpu::flags::Flag::Z);
    } else {
        cpu.p.clear(cpu::flags::Flag::Z);
    }

    addr_cycles
}

/* Index Register Instructions */

// LDX: Load Index Register X from Memory
// M -> X
pub fn ldx(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    update_zero_flag(cpu, mem);
    update_negative_flag(cpu, mem);
    cpu.x = mem;

    addr_cycles
}

// LDY: Load Index Register Y from Memory
// M -> Y
pub fn ldy(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let mem = cpu.load_memory(addr);

    update_zero_flag(cpu, mem);
    update_negative_flag(cpu, mem);
    cpu.y = mem;

    addr_cycles
}

// STX: Store Index Register X in Memory
// X -> M
pub fn stx(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.x;
    cpu.store_memory(addr, byte);
    addr_cycles
}

// STY: Store Index Register Y in Memory
// Y -> M
pub fn sty(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.y;
    cpu.store_memory(addr, byte);
    addr_cycles
}

// INX: Increment Index Register X by One
// X + 1 -> X
pub fn inx(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let res = cpu.x.wrapping_add(1);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.x = res;
    0
}

// INY: Increment Index Register Y by One
// Y + 1 -> Y
pub fn iny(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let res = cpu.y.wrapping_add(1);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.y = res;
    0
}

// DEX: Decrement Index Register X by One
// X + 1 -> X
pub fn dex(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let res = cpu.x.wrapping_sub(1);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.x = res;
    0
}

// DEY: Decrement Index Register Y by One
// Y + 1 -> Y
pub fn dey(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let res = cpu.y.wrapping_sub(1);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    cpu.y = res;
    0
}

// CPX - Compare Index Register X to Memory
// X - M
pub fn cpx(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.x;
    compare_instruction(cpu, load_addr, byte)
}

// CPY - Compare Index Register Y to Memory
// Y - M
pub fn cpy(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.y;
    compare_instruction(cpu, load_addr, byte)
}

// TAX: Transfer Accumulator to Index X
// A -> X
pub fn tax(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    update_negative_flag(cpu, byte);
    update_zero_flag(cpu, byte);
    cpu.x = byte;
    0
}

// TXA: Transfer Index X to Accumulator
// X -> A
pub fn txa(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.x;
    update_negative_flag(cpu, byte);
    update_zero_flag(cpu, byte);
    cpu.a = byte;
    0
}

// TAY: Transfer Accumulator to Index Y
// A -> Y
pub fn tay(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    update_negative_flag(cpu, byte);
    update_zero_flag(cpu, byte);
    cpu.y = byte;
    0
}

// TYA: Transfer Index Y to Accumulator
// Y -> A
pub fn tya(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.y;
    update_negative_flag(cpu, byte);
    update_zero_flag(cpu, byte);
    cpu.a = byte;
    0
}

/* 8. Stack Processing */

// JSR: Jump to Subroutine
// PC + 2v, (PC + 1) -> PCL, (PC + 2) -> PCH
pub fn jsr(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);

    // load_addr will leave the PC pointing at the next opcode.
    // JSR is actually supposed to store the previous address.
    cpu.pc -= 1;

    // Store PC on stack.
    let pc_high = (cpu.pc >> 8) as u8;
    let pc_low = cpu.pc as u8;
    cpu.stack_push(pc_high);
    cpu.stack_push(pc_low);

    // Jump to target address.
    cpu.pc = addr;

    addr_cycles
}

// RTS: Return from Subroutine
// PC^, INC PC
pub fn rts(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    // Load PC from stack.
    let pc_low = cpu.stack_pop();
    let pc_high = cpu.stack_pop();
    cpu.pc = util::combine_bytes(pc_high, pc_low);

    // JSR stores the address of the end of the JSR instruction.
    // So we need to increment the PC by 1 to point at the next opcode.
    cpu.pc += 1;

    0
}

// PHA: Push Accumulator on Stack
// Av
pub fn pha(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    cpu.stack_push(byte);
    0
}

// PLA: Pull Accumulator from Stack
// A^
pub fn pla(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.stack_pop();
    update_negative_flag(cpu, byte);
    update_zero_flag(cpu, byte);
    cpu.a = byte;
    0
}

// TXS: Transfer Index X to Stack Pointer
// X -> S
pub fn txs(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    cpu.sp = cpu.x;
    0
}

// TSX: Transfer Stack Pointer to Index X
// S -> X
pub fn tsx(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.sp;
    update_negative_flag(cpu, byte);
    update_zero_flag(cpu, byte);
    cpu.x = cpu.sp;
    0
}

// PHP: Push Processor Status on Stack
// Pv
pub fn php(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.p.as_byte();
    // Set the B flag to the value we push, but do not modify the status register.
    cpu.stack_push(byte | (cpu::flags::Flag::B as u8));
    0
}

// PLP: Pull Processor Status from Stack
// P^
// Make sure to ignore bits 4 and 5 since these are unused.
pub fn plp(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    load_status_from_stack(cpu);
    0
}

/* 8. Reset and Interrupt Considerations */

// RTI: Return from Interrupt
// ^P ^PC
pub fn rti(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    load_status_from_stack(cpu);

    let pcl = cpu.stack_pop();
    let pch = cpu.stack_pop();
    cpu.pc = util::combine_bytes(pch, pcl);
    0
}

// BRK: Break Command
// PC+2v (FFFE) -> PCL, (FFFF) -> PCH
pub fn brk(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    // Note: I'm not sure why it stores PC+2, but that's what the documentation says.
    // PC was incremented by 1 already for us before this function, so just add one.
    let pch = (cpu.pc >> 8) as u8;
    let pcl = cpu.pc as u8;
    cpu.stack_push(pch);
    cpu.stack_push(pcl + 1);

    cpu.p.set(cpu::flags::Flag::B);
    let byte = cpu.p.as_byte();
    //
    // Set the B flag to the value we push, but do not modify the status register.
    cpu.stack_push(byte | (cpu::flags::Flag::B as u8));

    // Load interrupt vector.
    let pcl = cpu.load_memory(0xFFFE);
    let pch = cpu.load_memory(0xFFFF);
    cpu.pc = util::combine_bytes(pch, pcl);
    0
}

/* 10. Shift and Memory Modify Instructions */

fn shift_set_flags(cpu: &mut cpu::CPU, res: u8, carry: bool) {
    if carry {
        cpu.p.set(cpu::flags::Flag::C);
    } else {
        cpu.p.clear(cpu::flags::Flag::C);
    }

    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
}

// The shift instructions have a special accumulator addressing mode which doesn't fit out model of
// addressing modes.  So we implement them as separate instructions.

// LSR: Logical Shift Right
pub fn lsr(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.load_memory(addr);
    let (res, carry) = util::shift_right(byte);
    shift_set_flags(cpu, res, carry);
    cpu.store_memory(addr, res);
    addr_cycles
}

pub fn lsra(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    let (res, carry) = util::shift_right(byte);
    shift_set_flags(cpu, res, carry);
    cpu.a = res;
    0
}

// ASL: Arithmetic Shift Left
pub fn asl(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.load_memory(addr);
    let (res, carry) = util::shift_left(byte);
    shift_set_flags(cpu, res, carry);
    cpu.store_memory(addr, res);
    addr_cycles
}

pub fn asla(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    let (res, carry) = util::shift_left(byte);
    shift_set_flags(cpu, res, carry);
    cpu.a = res;
    0
}

// ROR: Rotate Right
pub fn ror(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.load_memory(addr);
    let (res, carry) = util::rotate_right(byte, cpu.p.is_set(cpu::flags::Flag::C));
    shift_set_flags(cpu, res, carry);
    cpu.store_memory(addr, res);
    addr_cycles
}

pub fn rora(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    let (res, carry) = util::rotate_right(byte, cpu.p.is_set(cpu::flags::Flag::C));
    shift_set_flags(cpu, res, carry);
    cpu.a = res;
    0
}

// ROL: Rotate Left
pub fn rol(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.load_memory(addr);
    let (res, carry) = util::rotate_left(byte, cpu.p.is_set(cpu::flags::Flag::C));
    shift_set_flags(cpu, res, carry);
    cpu.store_memory(addr, res);
    addr_cycles
}

pub fn rola(cpu: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    let byte = cpu.a;
    let (res, carry) = util::rotate_left(byte, cpu.p.is_set(cpu::flags::Flag::C));
    shift_set_flags(cpu, res, carry);
    cpu.a = res;
    0
}

// INC: Increment Memory by One
// M + 1 -> M
pub fn inc(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.load_memory(addr);
    let res = byte.wrapping_add(1);
    cpu.store_memory(addr, res);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    addr_cycles
}

// DEC: Decrement Memory by One
// M - 1 -> M
pub fn dec(cpu: &mut cpu::CPU, load_addr: cpu::addressing::AddressingMode) -> u32 {
    let (addr, addr_cycles) = load_addr(cpu);
    let byte = cpu.load_memory(addr);
    let res = byte.wrapping_sub(1);
    cpu.store_memory(addr, res);
    update_zero_flag(cpu, res);
    update_negative_flag(cpu, res);
    addr_cycles
}

// NOP: No operation
pub fn nop(_: &mut cpu::CPU, _: cpu::addressing::AddressingMode) -> u32 {
    0
}
