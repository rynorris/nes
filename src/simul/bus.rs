use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct Bus<T> {
    val: Rc<RefCell<Option<T>>>,
}

impl <T : Copy> Bus<T> {
    pub fn send(&self, new: T) {
        self.val.replace(Some(new));
    }

    pub fn read(&self) -> Option<T> { *self.val.borrow() }

    pub fn clear(&self) {
        self.val.replace(None);
    }
}

pub fn new<T>() -> Bus<T> {
    Bus{ val: Rc::new(RefCell::new(None)) }
}

#[test]
fn test_bus_u8() {
    let bus: Bus<u8> = new();
    assert_eq!(bus.read(), None);
    bus.send(0xFA);
    assert_eq!(bus.read(), Some(0xFA));
    bus.clear();
    assert_eq!(bus.read(), None);
}

#[test]
fn test_clone_bus() {
    let bus: Bus<u8> = new();
    let clone = bus.clone();
    assert_eq!(bus.read(), None);
    assert_eq!(clone.read(), None);

    bus.send(0xFF);
    assert_eq!(bus.read(), Some(0xFF));
    assert_eq!(clone.read(), Some(0xFF));

    clone.clear();
    assert_eq!(bus.read(), None);
    assert_eq!(clone.read(), None);
}
