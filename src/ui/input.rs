use std::cell::RefCell;
use std::rc::Rc;

use emulator::io::event::{Event, EventBus, Key};
use ui::sdl2::event;
use ui::sdl2::keyboard::Keycode;

// Responsible for collecting SDL events and rebroadcasting them as internal events.
pub struct InputPump {
    event_pump: sdl2::EventPump,
    event_bus: Rc<RefCell<EventBus>>,
}

impl InputPump {
    pub fn new(event_pump: sdl2::EventPump, event_bus: Rc<RefCell<EventBus>>) -> InputPump {
        InputPump {
            event_pump,
            event_bus,
        }
    }

    pub fn pump(&mut self) {
        while let Some(e) = self.event_pump.poll_event() {
            let internal_event = convert_sdl_event_to_internal(e);

            if let Some(e) = internal_event {
                self.event_bus.borrow_mut().broadcast(e);
            }
        }
    }
}

fn convert_sdl_event_to_internal(event: event::Event) -> Option<Event> {
    match event {
        event::Event::KeyDown { keycode, .. } => keycode
            .and_then(|k| convert_sdl_keycode_to_internal(k))
            .map(|k| Event::KeyDown(k)),
        event::Event::KeyUp { keycode, .. } => keycode
            .and_then(|k| convert_sdl_keycode_to_internal(k))
            .map(|k| Event::KeyUp(k)),
        _ => None,
    }
}

fn convert_sdl_keycode_to_internal(keycode: Keycode) -> Option<Key> {
    match keycode {
        Keycode::A => Some(Key::A),
        Keycode::B => Some(Key::B),
        Keycode::C => Some(Key::C),
        Keycode::D => Some(Key::D),
        Keycode::E => Some(Key::E),
        Keycode::F => Some(Key::F),
        Keycode::G => Some(Key::G),
        Keycode::H => Some(Key::H),
        Keycode::I => Some(Key::I),
        Keycode::J => Some(Key::J),
        Keycode::K => Some(Key::K),
        Keycode::L => Some(Key::L),
        Keycode::M => Some(Key::M),
        Keycode::N => Some(Key::N),
        Keycode::O => Some(Key::O),
        Keycode::P => Some(Key::P),
        Keycode::Q => Some(Key::Q),
        Keycode::S => Some(Key::S),
        Keycode::T => Some(Key::T),
        Keycode::U => Some(Key::U),
        Keycode::V => Some(Key::V),
        Keycode::W => Some(Key::W),
        Keycode::X => Some(Key::X),
        Keycode::Y => Some(Key::Y),
        Keycode::Z => Some(Key::Z),

        Keycode::Backquote => Some(Key::Backquote),
        Keycode::Num1 => Some(Key::Num1),
        Keycode::Num2 => Some(Key::Num2),
        Keycode::Num3 => Some(Key::Num3),
        Keycode::Num4 => Some(Key::Num4),
        Keycode::Num5 => Some(Key::Num5),
        Keycode::Num6 => Some(Key::Num6),
        Keycode::Num7 => Some(Key::Num7),
        Keycode::Num8 => Some(Key::Num8),
        Keycode::Num9 => Some(Key::Num9),
        Keycode::Num0 => Some(Key::Num0),
        Keycode::Minus => Some(Key::Minus),
        Keycode::Equals => Some(Key::Equals),
        Keycode::Backspace => Some(Key::Backspace),

        Keycode::Up => Some(Key::Up),
        Keycode::Down => Some(Key::Down),
        Keycode::Left => Some(Key::Left),
        Keycode::Right => Some(Key::Right),

        Keycode::Escape => Some(Key::Escape),
        Keycode::Return => Some(Key::Return),
        Keycode::Tab => Some(Key::Tab),
        Keycode::Space => Some(Key::Space),

        Keycode::LShift => Some(Key::Shift),
        Keycode::LCtrl => Some(Key::Control),

        _ => None
    }
}
