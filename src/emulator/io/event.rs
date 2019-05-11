use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

// Framework agnostic internal event types.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Backquote,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Up,
    Down,
    Left,
    Right,
    Minus,
    Equals,
    Backspace,
    Escape,
    Return,
    Tab,
    Space,
    Shift,
    Control,
}

pub trait EventHandler {
    fn handle_event(&mut self, event: Event);
    fn handle_event_with_dispatch(&mut self, _dispatch: &FnOnce(Event) -> (), event: Event) {
        self.handle_event(event);
    }
}

impl<H: EventHandler> EventHandler for Rc<RefCell<H>> {
    fn handle_event(&mut self, event: Event) {
        self.borrow_mut().handle_event(event);
    }
}

pub struct EventBus {
    event_handlers: Vec<Box<dyn EventHandler>>,
    event_queue: VecDeque<Event>,
}

impl EventBus {
    pub fn new() -> EventBus {
        EventBus {
            event_handlers: vec![],
            event_queue: VecDeque::new(),
        }
    }

    pub fn broadcast(&mut self, event: Event) {
        self.event_queue.push_back(event);
        self.flush_queue();
    }

    fn flush_queue(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: Event) {
        let deque = &mut self.event_queue;
        for handler in self.event_handlers.iter_mut() {
            handler.handle_event_with_dispatch(&|e| deque.push_back(e), event);
        }
    }

    pub fn register(&mut self, handler: Box<dyn EventHandler>) {
        self.event_handlers.push(handler);
    }
}
