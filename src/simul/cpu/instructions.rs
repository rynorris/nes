use simul::cpu;

type Operation = fn(&mut cpu::CPU, &cpu::addressing::AddressingMode);

pub fn lookup_opcode(opcode: u8) -> (Operation, cpu::addressing::AddressingMode) {
    match opcode {
        _ => panic!("Unknown opcode: {:X}", opcode)
    }
}
