// ----------------------------------------------------------------------------
pub enum Key {
    Exit = 0,
    LookLeft = 1,
    LookRight = 2,
    LookUp = 3,
    LookDown = 4,
    MoveForward = 5,
    MoveBackward = 6,
    StrafeLeft = 7,
    StrafeRight = 8,
}

// ----------------------------------------------------------------------------
pub enum Event {
    MouseMove { x: i32, y: i32 },
    ButtonDown { button: u32 },
    ButtonUp { button: u32 },
    Wheel { delta: i32 },
    KeyDown { key: Key },
    KeyUp { key: Key },
}

// ----------------------------------------------------------------------------
pub struct Input {
    events: Vec<Event>,
    states: [u8; 256],
}

impl Default for Input {
    fn default() -> Input {
        Input::new()
    }
}

impl Input {
    pub fn new() -> Input {
        Input {
            events: Vec::new(),
            states: [0; 256],
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        let key = key as usize;
        if let Some(&s) = self.states.get(key) {
            s != 0
        } else {
            false
        }
    }

    pub fn set_state(&mut self, key: Key, state: u8) {
        let key = key as usize;
        if let Some(s) = self.states.get_mut(key) {
            *s = state;
        }
    }
}
