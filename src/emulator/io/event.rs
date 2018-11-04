use std::cell::RefCell;
use std::rc::Rc;

// Framework agnostic internal event types.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0,
    Up, Down, Left, Right,
    Minus, Equals,
    Escape, Return, Tab, Space,
}

pub trait EventHandler {
    fn handle_event(&mut self, event: Event);
}

impl <H : EventHandler> EventHandler for Rc<RefCell<H>> {
    fn handle_event(&mut self, event: Event) {
        self.borrow_mut().handle_event(event);
    }
}
