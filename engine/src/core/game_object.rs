use crate::v2d::{affine4x4, m4x4::M4x4, v4::V4};

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Copy, Clone)]
pub struct Transform {
    pub position: V4,
    pub rotation: V4,
}

// ----------------------------------------------------------------------------
impl From<Transform> for M4x4 {
    fn from(tx: Transform) -> Self {
        affine4x4::translate(&tx.position) * affine4x4::rotate_x1(tx.rotation.x1())
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Clone)]
pub struct GameObject {
    pub name: String,
    pub children: Vec<GameObject>,
    pub transform: Transform,
    pub pipe_id: u32,
    pub mesh_id: u32,
    pub material_id: u32,
}
