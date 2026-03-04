use crate::v2d::{q::Q, v3::V3};
use crate::x2d::{
    BodyHandle, Material,
    collider::{Collider, ColliderHandle, ColliderShape, ContactPoint, detect},
    contact_constraint::{ContactConstraint, ContactPair},
    joint_constraint::{JointConstraint, JointHandle},
    mass::Mass,
    rigid_body::RigidBody,
};

// ----------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct World {
    bodies: Vec<RigidBody>,
    colliders: Vec<Collider>,
    contacts: Vec<ContactConstraint>,
    joints: Vec<JointConstraint>,
}

// ----------------------------------------------------------------------------
impl World {
    // ------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            colliders: Vec::new(),
            contacts: Vec::new(),
            joints: Vec::new(),
        }
    }
}
