use crate::core::{camera::Camera, player::Player, terrain::Terrain};
use crate::error::Result;
use crate::v2d::{affine4x4, m4x4::M4x4, v4::V4};

// ----------------------------------------------------------------------------
#[rustfmt::skip]
#[repr(u8)]
pub enum EntityFlag {
	Empty		= 0,
	Valid		= 1 << 0,
	Visible 	= 1 << 1,
	Active		= 1 << 2,
	Static		= 1 << 3,
	Mesh		= 1 << 4,
}

// ----------------------------------------------------------------------------
impl EntityFlag {
    pub fn contains(flags: u8, flag: EntityFlag) -> bool {
        (flags & flag as u8) != 0
    }
    pub fn set(flags: u8, flag: EntityFlag) -> u8 {
        flags | flag as u8
    }
    pub fn clear(flags: u8, flag: EntityFlag) -> u8 {
        flags & !(flag as u8)
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(usize);

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Copy, Clone)]
pub struct Transform {
    pub position: V4,
    pub rotation: V4,
}

// ----------------------------------------------------------------------------
impl From<Transform> for M4x4 {
    fn from(tx: Transform) -> Self {
        affine4x4::translate(&tx.position) * affine4x4::rotate_x0(tx.rotation.x0())
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct Entity {
    pub name: String,
    pub children: Vec<EntityId>,
    pub transform: Transform,
    pub pipe_id: u32,
    pub mesh_id: u32,
    pub material_id: u32,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct EntityComponentSystem {
    flags: Vec<u8>,
    entities: Vec<Entity>,
    free_ids: Vec<usize>,
}

// ----------------------------------------------------------------------------
impl EntityComponentSystem {
    pub fn add_entity(
        &mut self,
        flags: u8,
        name: String,
        transform: Transform,
        pipe_id: u32,
        mesh_id: u32,
        material_id: u32,
    ) -> EntityId {
        let id = self.free_ids.pop().unwrap_or(self.flags.len());
        let flags = EntityFlag::Valid as u8 | flags;
        let entity = Entity {
            name,
            children: Vec::new(),
            transform,
            pipe_id,
            mesh_id,
            material_id,
        };

        if id >= self.flags.len() {
            self.flags.push(flags);
            self.entities.push(entity);
        } else {
            self.flags[id] = flags;
            self.entities[id] = entity;
        }

        EntityId(id)
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.flags[id.0] = EntityFlag::Empty as u8;
        self.free_ids.push(id.0);
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(id.0)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(id.0)
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct World {
    // The World struct can hold game state, entities, components, etc.
    ecs: EntityComponentSystem,
    terrain: Terrain,
    player: Player,
    camera: Camera,
    t: std::time::Duration,
}

impl World {
    pub fn new() -> Self {
        let mut ecs = EntityComponentSystem::default();
        let terrain = Terrain::default();
        let player = Player::default();
        let camera = Camera::new(V4::new([0.0, 2.0, 4.0, 1.0]), V4::new([0.0, 0.0, 0.0, 1.0]));
        let t = std::time::Duration::ZERO;

        let transform = Transform {
            position: V4::new([0.0, 2.0, 0.0, 1.0]),
            rotation: V4::new([0.0, 0.0, 0.0, 1.0]),
        };
        ecs.add_entity(
            EntityFlag::Mesh as u8,
            "cube".to_string(),
            transform,
            0,
            0,
            7,
        );

        World {
            ecs,
            terrain,
            camera,
            player,
            t,
        }
    }

    pub fn mesh_entities(&self) -> impl Iterator<Item = &Entity> {
        self.ecs
            .flags
            .iter()
            .zip(self.ecs.entities.iter())
            .filter_map(|(flags, entity)| {
                if EntityFlag::contains(*flags, EntityFlag::Mesh) {
                    Some(entity)
                } else {
                    None
                }
            })
    }

    pub fn add_mesh_entity(
        &mut self,
        name: String,
        transform: Transform,
        pipe_id: u32,
        mesh_id: u32,
        material_id: u32,
    ) -> EntityId {
        self.ecs.add_entity(
            EntityFlag::Mesh as u8,
            name,
            transform,
            pipe_id,
            mesh_id,
            material_id,
        )
    }

    pub fn update(&mut self, dt: &std::time::Duration) -> Result<()> {
        self.t += *dt;
        self.terrain
            .update(&mut self.ecs, &V4::default(), &V4::default())?;
        self.camera.update(dt)?;
        self.player.update(&mut self.ecs, dt)?;
        self.ecs
            .flags
            .iter_mut()
            .zip(self.ecs.entities.iter_mut())
            .filter_map(|(flags, entity)| {
                if EntityFlag::contains(*flags, EntityFlag::Mesh) {
                    Some(entity)
                } else {
                    None
                }
            })
            .take(1)
            .for_each(|entity| {
                entity.transform.rotation = V4::new([self.t.as_secs_f32(), 0.0, 0.0, 0.0]);
            });
        Ok(())
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.camera.move_forward(distance);
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.camera.move_backward(distance);
    }

    pub fn strafe_left(&mut self, distance: f32) {
        self.camera.strafe_left(distance);
    }

    pub fn strafe_right(&mut self, distance: f32) {
        self.camera.strafe_right(distance);
    }

    pub fn pan_camera(&mut self, x: f32) {
        self.camera.pan(x);
    }

    pub fn tilt_camera(&mut self, y: f32) {
        self.camera.tilt(y);
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
