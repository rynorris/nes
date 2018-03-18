use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct Bus<T> {
    val: Rc<RefCell<T>>,
}

impl <T : Copy> Bus<T> {
    pub fn load(&self, new: T) {
        self.val.replace(new);
    }

    pub fn read(&self) -> T { *self.val.borrow() }
}

pub fn new<T>(init: T) -> Bus<T> {
    Bus{ val: Rc::new(RefCell::new(init)) }
}

#[test]
fn test_bus_u8() {
    let bus: Bus<u8> = new(0x00);
    assert_eq!(bus.read(), 0x00);
    bus.load(0xFA);
    assert_eq!(bus.read(), 0xFA);
}
