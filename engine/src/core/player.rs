use crate::core::game_object::GameObject;
use crate::core::input;
use crate::error::Result;
use crate::v2d::{r2::R2, v2::V2, v4::V4};

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Player {
    pub game_object: GameObject,
    pub velocity: V2,
    pub rotation: R2,
    pub speed: f32,
}

// ----------------------------------------------------------------------------
impl Player {
    pub fn new() -> Self {
        Self {
            game_object: GameObject::default(),
            velocity: V2::default(),
            rotation: R2::default(),
            speed: 5.0,
        }
    }

    pub fn update(&mut self, dt: &std::time::Duration, input: &input::State) -> Result<()> {
        if input.is_pressed(input::Key::MoveForward) {
            let direction = self.rotation.x_axis();
            self.velocity = direction * self.speed;
        }

        let dt = dt.as_secs_f32();
        let displacement = self.velocity * dt;
        let displacement = V4::new([displacement.x0(), displacement.x1(), 0.0, 0.0]);
        self.game_object.transform.position += displacement;

        let rotation = self.rotation.get();
        let rotation = V4::new([0.0, 0.0, rotation, 0.0]);
        self.game_object.transform.rotation = rotation;

        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
