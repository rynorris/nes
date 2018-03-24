const ADDRESS_SPACE: usize = 65536;

pub fn new() -> RAM {
    RAM{
        memory: [0; ADDRESS_SPACE],
    }
}

pub struct RAM {
    memory: [u8; ADDRESS_SPACE],
}

impl RAM {
    pub fn load(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn store(&mut self, address: u16, byte: u8) {
        self.memory[address as usize] = byte
    }

    pub fn debug_print(&self, start_addr: u16, num_bytes: u16) {
        let end_addr = start_addr + num_bytes;
        println!("RAM [{:?}..{:?}]: {:?}", start_addr, end_addr, &self.memory[(start_addr as usize) .. (end_addr as usize)]);
    }
}

#[test]
fn test_get_and_set() {
    let mut ram = new();
    ram.store(1234, 23);
    assert_eq!(ram.load(1234), 23);
}
