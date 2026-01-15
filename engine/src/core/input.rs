// ----------------------------------------------------------------------------
pub enum Event {
    MouseMove { x: i32, y: i32 },
    ButtonDown { button: u32 },
    ButtonUp { button: u32 },
    Wheel { delta: i32 },
    KeyDown { key: u32 },
    KeyUp { key: u32 },
}

// ----------------------------------------------------------------------------
pub enum Key {
    MoveForward = 0,
    MoveBackward = 1,
    StrafeLeft = 2,
    StrafeRight = 3,
}

// ----------------------------------------------------------------------------
use windows::Win32::UI::Input::KeyboardAndMouse;

const VK_W: usize = KeyboardAndMouse::VK_W.0 as usize;
const VK_A: usize = KeyboardAndMouse::VK_A.0 as usize;
const VK_S: usize = KeyboardAndMouse::VK_S.0 as usize;
const VK_D: usize = KeyboardAndMouse::VK_D.0 as usize;

// ----------------------------------------------------------------------------
const KEY_MAP: [usize; 4] = [VK_W, VK_S, VK_A, VK_D];

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
        KEY_MAP
            .get(key as usize)
            .map(|v| self.states[*v] != 0)
            .unwrap_or(false)
    }

    pub fn set_state(&mut self, key: usize, state: u8) {
        if let Some(s) = self.states.get_mut(key) {
            *s = state;
        }
    }
}
