mod instructions;
mod flags;

use simul::bus;

// CPU Implemented as a state machine.
pub struct CPU {
    // Accumulator
    pub a: u8,

    // X Index Register
    pub x: u8,

    // Y Index Register
    pub y: u8,

    // Stack Pointer
    pub sp: u8,

    // Program Counter
    pub pc: u16,

    // Processor Flags NV_BDIZC
    p: flags::ProcessorFlags,

    // Current state.
    state: State,

    // Current addressing mode.
    addressing_mode: AddressingMode,

    // Address bus
    address_bus: bus::Bus<u16>,

    // Data bus
    data_bus: bus::Bus<u8>,
}

pub fn new(address_bus: bus::Bus<u16>, data_bus: bus::Bus<u8>) -> CPU {
    CPU {
        a: 0,
        x: 0,
        y: 0,
        sp: 0,
        pc: 0,
        p: flags::new(),
        state: State::Init,
        addressing_mode: AddressingMode::Implied,
        address_bus,
        data_bus,
    }
}

#[derive(Debug)]
enum State {
    Init,
    StartOp,
    LoadedOperand1,
    LoadedOperand2,
}

#[derive(Debug)]
pub enum AddressingMode {
    // Implied: no operand.
    Implied,

    // Absolute: two byte operand indicates memory address.
    Absolute,

    // Immediate: one byte literal operand.
    Immediate,

    // Zero page: one byte operand indicates address in page 0 of memory.
    ZeroPage,

    // Relative: one byte operand indicates address relative to PC.
    Relative,

    // Absolute indexed: same as absolute addressing, but adds an index register to the
    // address.
    AbsoluteIndexedX,
    AbsoluteIndexedY,

    // Zero page indexed: same as zero page, but adds an index register to the address.
    // Only supported for index X.
    // If the resulting value is greated than 255, the address wraps within page 0.
    ZeroPageIndexedX,

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
    IndexedIndirect,
    IndirectIndexed,
    IndirectAbsolute,
}

impl CPU {
    pub fn do_cycle(&mut self) {
        let new_state = match self.state {
            State::Init => self.init(),
            _ => panic!("Unimplemented CPU state {:?}", self.state),
        };
        self.state = new_state;
    }

    // Initial state just prepares to load the first operation.
    fn init(&mut self) -> State {
        self.address_bus.load(self.pc);
        State::StartOp
    }
}
