use crate::core::game_object::{GameObject, Transform};
use crate::error::Result;
use crate::v2d::v4::V4;
use std::collections::HashMap;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Terrain {
    width: i32,
    height: i32,
    visible_chunks: HashMap<(i32, i32), GameObject>,
}

// ----------------------------------------------------------------------------
impl Terrain {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            visible_chunks: HashMap::new(),
        }
    }

    pub fn update(&mut self, _cam_pos: &V4, _cam_dir: &V4) -> Result<()> {
        for x in 0..self.width {
            for y in 0..self.height {
                let key = (x, y);
                self.visible_chunks
                    .entry(key)
                    .or_insert_with(|| GameObject {
                        name: format!("terrain_{x}_{y}"),
                        transform: Transform {
                            position: V4::new([x as f32 * 10.0, 0.0, y as f32 * 10.0, 1.0]),
                            rotation: V4::default(),
                        },
                        pipe_id: 0,
                        mesh_id: 1,
                        material_id: 0,
                        ..Default::default()
                    });
            }
        }
        Ok(())
    }

    pub fn visible_chunks(&self) -> Vec<GameObject> {
        self.visible_chunks.values().cloned().collect()
    }
}

// ----------------------------------------------------------------------------
impl Default for Terrain {
    fn default() -> Self {
        Self::new(4, 4)
    }
}
