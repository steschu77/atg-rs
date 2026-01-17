use crate::core::game_object::GameObject;
use crate::core::{camera::Camera, input, player::Player, terrain::Terrain};
use crate::error::Result;
use crate::v2d::v4::V4;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct World {
    // The World struct can hold game state, entities, components, etc.
    terrain: Terrain,
    player: Player,
    camera: Camera,
    t: std::time::Duration,
}

impl World {
    pub fn new() -> Self {
        let terrain = Terrain::default();
        let player = Player::default();
        let camera = Camera::new(V4::new([0.0, 2.0, 4.0, 1.0]), V4::new([0.0, 0.0, 0.0, 1.0]));
        let t = std::time::Duration::ZERO;

        World {
            terrain,
            camera,
            player,
            t,
        }
    }

    pub fn update(
        &mut self,
        dt: &std::time::Duration,
        events: &input::Events,
        state: &input::State,
    ) -> Result<()> {
        self.t += *dt;
        self.terrain.update(&V4::default(), &V4::default())?;
        self.camera.update(dt, events)?;
        self.player.update(dt, state)?;
        Ok(())
    }

    pub fn objects(&self) -> Vec<GameObject> {
        let mut objects = self.terrain.visible_chunks();
        objects.push(self.player.game_object.clone());
        objects
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

// ----------------------------------------------------------------------------
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
