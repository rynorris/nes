use nes::emulator::io::event;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Event {
    event_type: EventType,
    key: Option<Key>,
}

#[wasm_bindgen]
impl Event {
    pub fn key_down(key: Key) -> Event {
        Event {
            event_type: EventType::KeyDown,
            key: Some(key),
        }
    }

    pub fn key_up(key: Key) -> Event {
        Event {
            event_type: EventType::KeyUp,
            key: Some(key),
        }
    }
}

// WASM-facing copies of internal event types.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    KeyDown,
    KeyUp,
}

#[wasm_bindgen]
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

pub fn convert_wasm_event_to_internal(event: Event) -> event::Event {
    match event.event_type {
        EventType::KeyDown => {
            event::Event::KeyDown(convert_wasm_key_to_internal(event.key.unwrap()))
        }
        EventType::KeyUp => event::Event::KeyUp(convert_wasm_key_to_internal(event.key.unwrap())),
    }
}

fn convert_wasm_key_to_internal(key: Key) -> event::Key {
    match key {
        Key::A => event::Key::A,
        Key::B => event::Key::B,
        Key::C => event::Key::C,
        Key::D => event::Key::D,
        Key::E => event::Key::E,
        Key::F => event::Key::F,
        Key::G => event::Key::G,
        Key::H => event::Key::H,
        Key::I => event::Key::I,
        Key::J => event::Key::J,
        Key::K => event::Key::K,
        Key::L => event::Key::L,
        Key::M => event::Key::M,
        Key::N => event::Key::N,
        Key::O => event::Key::O,
        Key::P => event::Key::P,
        Key::Q => event::Key::Q,
        Key::R => event::Key::R,
        Key::S => event::Key::S,
        Key::T => event::Key::T,
        Key::U => event::Key::U,
        Key::V => event::Key::V,
        Key::W => event::Key::W,
        Key::X => event::Key::X,
        Key::Y => event::Key::Y,
        Key::Z => event::Key::Z,
        Key::Backquote => event::Key::Backquote,
        Key::Num1 => event::Key::Num1,
        Key::Num2 => event::Key::Num2,
        Key::Num3 => event::Key::Num3,
        Key::Num4 => event::Key::Num4,
        Key::Num5 => event::Key::Num5,
        Key::Num6 => event::Key::Num6,
        Key::Num7 => event::Key::Num7,
        Key::Num8 => event::Key::Num8,
        Key::Num9 => event::Key::Num9,
        Key::Num0 => event::Key::Num0,
        Key::Up => event::Key::Up,
        Key::Down => event::Key::Down,
        Key::Left => event::Key::Left,
        Key::Right => event::Key::Right,
        Key::Minus => event::Key::Minus,
        Key::Equals => event::Key::Equals,
        Key::Backspace => event::Key::Backspace,
        Key::Escape => event::Key::Escape,
        Key::Return => event::Key::Return,
        Key::Tab => event::Key::Tab,
        Key::Space => event::Key::Space,
        Key::Shift => event::Key::Shift,
        Key::Control => event::Key::Control,
    }
}
