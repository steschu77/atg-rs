// ----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
#[rustfmt::skip]
pub enum Key {
    k_Escape,
    k_F1, k_F2, k_F3, k_F4, k_F5, k_F6, k_F7, k_F8, k_F9, k_F10, k_F11, k_F12,
    k_Return, k_Space, k_Backspace, k_Tab,
    k_Insert, k_Delete, k_Home, k_End, k_PageUp, k_PageDown,
    k_Up, k_Down, k_Left, k_Right,
    k_LeftShift, k_LeftCtrl, k_LeftAlt, k_LeftSuper,
    k_RightShift, k_RightCtrl, k_RightAlt, k_RightSuper,
    k_0, k_1, k_2, k_3, k_4, k_5, k_6, k_7, k_8, k_9,
    k_A, k_B, k_C, k_D, k_E, k_F, k_G, k_H, k_I, k_J,
    k_K, k_L, k_M, k_N, k_O, k_P, k_Q, k_R, k_S, k_T,
    k_U, k_V, k_W, k_X, k_Y, k_Z,
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
