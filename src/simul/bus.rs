use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct AddressBus {
    val: Rc<RefCell<u16>>,
}

impl AddressBus {
    pub fn load_high_byte(&self, byte: u8) {
        let new = (*self.val.borrow() & 0x00FF) | ((byte as u16) << 8);
        self.val.replace(new);
    }

    pub fn load_low_byte(&self, byte: u8) {
        let new = (*self.val.borrow() & 0xFF00) | (byte as u16);
        self.val.replace(new);
    }

    pub fn read(&self) -> u16 { *self.val.borrow() }
}

pub fn new_address_bus() -> AddressBus {
    AddressBus{ val: Rc::new(RefCell::new(0)) }
}

#[derive(Clone)]
pub struct DataBus {
    val: Rc<RefCell<u8>>,
}

impl DataBus {
    pub fn load(&self, byte: u8) {
        self.val.replace(byte);
    }

    pub fn read(&self) -> u8 { *self.val.borrow() }
}

pub fn new_data_bus() -> DataBus {
    DataBus{ val: Rc::new(RefCell::new(0)) }
}

#[test]
fn test_address_bus() {
    let bus: AddressBus = new_address_bus();
    bus.load_low_byte(0xAB);
    assert_eq!(bus.read(), 0x00AB);
    bus.load_high_byte(0x98);
    assert_eq!(bus.read(), 0x98AB);
    bus.load_high_byte(0x12);
    assert_eq!(bus.read(), 0x12AB);
    bus.load_low_byte(0x34);
    assert_eq!(bus.read(), 0x1234);
}

#[test]
fn test_data_bus() {
    let bus: DataBus = new_data_bus();
    assert_eq!(bus.read(), 0x00);
    bus.load(0xFA);
    assert_eq!(bus.read(), 0xFA);
}
