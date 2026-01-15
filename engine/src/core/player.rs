use crate::core::world::{EntityComponentSystem, EntityFlag, EntityId, Transform};
use crate::error::Result;
use crate::v2d::v4::V4;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Player {
    entity_id: Option<EntityId>,
}

// ----------------------------------------------------------------------------
impl Player {
    pub fn new() -> Self {
        Self { entity_id: None }
    }

    pub fn update(
        &mut self,
        ecs: &mut EntityComponentSystem,
        dt: &std::time::Duration,
    ) -> Result<()> {
        if self.entity_id.is_none() {
            self.entity_id = Some(ecs.add_entity(
                EntityFlag::Mesh as u8,
                "player".to_string(),
                Transform {
                    position: V4::new([2.0, 2.0, 1.0, 1.0]),
                    rotation: V4::new([0.0, 0.0, 0.0, 1.0]),
                },
                0,
                0,
                1,
            ));
        }

        let dt = dt.as_secs_f32();
        let direction = V4::new([1.0, 0.0, 0.0, 0.0]);

        if let Some(entity_id) = &self.entity_id {
            if let Some(entity) = ecs.get_mut(*entity_id) {
                entity.transform.position += direction * dt;
            }
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
