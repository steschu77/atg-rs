use crate::core::component::{Component, Context};
use crate::core::game_object::{GameObject, Transform};
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
impl Component for Player {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();

        if ctx.state.is_pressed(input::Key::TurnLeft) {
            self.rotation -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(input::Key::TurnRight) {
            self.rotation += TURN_SPEED * dt;
        }

        if ctx.state.is_pressed(input::Key::MoveForward) {
            let direction = self.rotation.x_axis();
            self.velocity = direction * self.speed;
        } else {
            self.velocity = V2::default();
        }

        let position = V2::new([
            self.game_object.transform.position.x0(),
            self.game_object.transform.position.x2(),
        ]);
        let displacement = self.velocity * dt;
        let position = position + displacement;

        let height = ctx.terrain.height_at(position.x0(), position.x1());

        self.game_object.transform.position = V4::new([position.x0(), height, position.x1(), 1.0]);

        let rotation = self.rotation.get();
        let rotation = V4::new([0.0, rotation, 0.0, 0.0]);
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

// ----------------------------------------------------------------------------
impl Player {
    pub fn new() -> Self {
        Self {
            game_object: GameObject {
                name: String::from("player"),
                transform: Transform {
                    position: V4::new([0.0, 0.0, 0.0, 1.0]),
                    rotation: V4::default(),
                },
                pipe_id: 0,
                mesh_id: 0,
                material_id: 0,
                ..Default::default()
            },
            velocity: V2::default(),
            rotation: R2::default(),
            speed: 5.0,
        }
    }
}
