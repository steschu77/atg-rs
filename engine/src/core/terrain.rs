use crate::core::world::{EntityComponentSystem, EntityFlag, EntityId, Transform};
use crate::error::Result;
use crate::v2d::v4::V4;
use std::collections::HashMap;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Terrain {
    width: i32,
    height: i32,
    entities: HashMap<(i32, i32), EntityId>,
}

// ----------------------------------------------------------------------------
impl Terrain {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            entities: HashMap::new(),
        }
    }

    pub fn update(
        &mut self,
        ecs: &mut EntityComponentSystem,
        _cam_pos: &V4,
        _cam_dir: &V4,
    ) -> Result<()> {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.entities.contains_key(&(x, y)) {
                    continue; // Skip if the entity already exists
                }
                let entity_id = ecs.add_entity(
                    EntityFlag::Mesh as u8,
                    format!("terrain_{x}_{y}"),
                    Transform {
                        position: V4::new([(2 * x) as f32, 0.0, -(2 * y) as f32, 1.0]),
                        rotation: V4::new([0.0, 0.0, 0.0, 1.0]),
                    },
                    0,
                    1,
                    11 * x as u32 + 43 * y as u32,
                );
                self.entities.insert((x, y), entity_id);
            }
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Default for Terrain {
    fn default() -> Self {
        Self::new(4, 4)
    }
}
