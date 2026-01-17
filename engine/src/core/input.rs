// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    MouseMove { x: i32, y: i32 },
    ButtonDown { button: u32 },
    ButtonUp { button: u32 },
    Wheel { delta: i32 },
    KeyDown { key: Key },
    KeyUp { key: Key },
}

// ----------------------------------------------------------------------------
pub type Events = Vec<Event>;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    keys: [u8; 256],
}

// ----------------------------------------------------------------------------
impl State {
    pub fn is_pressed(&self, key: Key) -> bool {
        let key = key as usize;
        self.keys.get(key).is_some_and(|&s| s != 0)
    }
}

// ----------------------------------------------------------------------------
impl Default for State {
    fn default() -> State {
        State { keys: [0; 256] }
    }
}

// ----------------------------------------------------------------------------
pub struct Input {
    events: Events,
    state: State,
}

// ----------------------------------------------------------------------------
impl Default for Input {
    fn default() -> Input {
        Input::new()
    }
}

// ----------------------------------------------------------------------------
impl Input {
    pub fn new() -> Input {
        Input {
            events: Vec::new(),
            state: State { keys: [0; 256] },
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn take_events(&mut self) -> Events {
        std::mem::take(&mut self.events)
    }

    pub fn set_state(&mut self, key: Key, state: u8) {
        let key = key as usize;
        if let Some(s) = self.state.keys.get_mut(key) {
            *s = state;
        }
    }

    pub fn take_state(&self) -> State {
        self.state.clone()
    }
}
