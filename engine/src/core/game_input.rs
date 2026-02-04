use crate::core::input::{Key, State};

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameKey {
    // System
    Menu = 0,

    // Camera / look
    LookLeft = 1,
    LookRight = 2,
    LookUp = 3,
    LookDown = 4,
    LookBack = 5,
    CameraToggle = 6,

    // On-foot movement
    MoveForward = 7,
    MoveBackward = 8,
    StrafeLeft = 9,
    StrafeRight = 10,
    Jump = 11,
    Crouch = 12,

    // Interaction
    Interact = 13,
    UseItem = 14,
    Inventory = 15,
    Map = 16,

    // Vehicle controls
    Accelerate = 17,
    Brake = 18,
    SteerLeft = 19,
    SteerRight = 20,
    Handbrake = 21,
    Horn = 22,
    Lights = 23,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct InputContext {
    mapping: [Key; GameKey::Lights as usize + 1],
    state: State,
}

// ----------------------------------------------------------------------------
impl Default for InputContext {
    fn default() -> Self {
        Self {
            mapping: [
                Key::k_Escape,    // Menu
                Key::k_Left,      // LookLeft
                Key::k_Right,     // LookRight
                Key::k_Up,        // LookUp
                Key::k_Down,      // LookDown
                Key::k_Backspace, // LookBack
                Key::k_C,         // CameraToggle
                Key::k_W,         // MoveForward
                Key::k_S,         // MoveBackward
                Key::k_A,         // StrafeLeft
                Key::k_D,         // StrafeRight
                Key::k_Space,     // Jump
                Key::k_LeftCtrl,  // Crouch
                Key::k_E,         // Interact
                Key::k_F,         // UseItem
                Key::k_I,         // Inventory
                Key::k_M,         // Map
                Key::k_W,         // Accelerate
                Key::k_S,         // Brake
                Key::k_A,         // SteerLeft
                Key::k_D,         // SteerRight
                Key::k_Space,     // Handbrake
                Key::k_H,         // Horn
                Key::k_L,         // Lights
            ],
            state: State::default(),
        }
    }
}

// ----------------------------------------------------------------------------
impl InputContext {
    pub fn update_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn is_pressed(&self, key: GameKey) -> bool {
        let key = self.mapping.get(key as usize);
        key.is_some_and(|&k| self.state.is_pressed(k))
    }
}
