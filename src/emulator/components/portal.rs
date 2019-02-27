use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Portal<T> {
    value: Arc<Mutex<T>>,
}

impl <T> Portal<T> {
    pub fn new(initial: T) -> Portal<T> {
        Portal {
            value: Arc::new(Mutex::new(initial)),
        }
    }

    pub fn consume<F : FnOnce(&mut T) -> ()>(&self, action: F) {
        let mut v = self.value.lock().expect("Could not lock mutex");
        action(&mut *v);
    }
}

unsafe impl <T> Send for Portal<T> {}
unsafe impl <T> Sync for Portal<T> {}
