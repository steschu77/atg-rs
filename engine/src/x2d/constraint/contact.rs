use crate::util::obj_pool::ObjPool;
use crate::x2d::BodyId;
use crate::x2d::constraint::tire_contact::{TireContact, TireContext};
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Contact {
    Tire { body: BodyId, contact: TireContact },
}

// ----------------------------------------------------------------------------
impl Contact {
    // ------------------------------------------------------------------------
    pub fn new_tire(body: BodyId, context: TireContext) -> Self {
        Self::Tire {
            body,
            contact: TireContact::new(context),
        }
    }

    // ------------------------------------------------------------------------
    pub fn update(&mut self, contact: TireContext) {
        match self {
            Self::Tire { contact: c, .. } => {
                c.update(contact);
            }
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, bodies: &mut ObjPool<RigidBody>, dt: f32) {
        match self {
            Self::Tire { body, contact } => {
                if let Some(body) = bodies.get(*body) {
                    contact.pre_step(body, dt);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, bodies: &mut ObjPool<RigidBody>) {
        match self {
            Self::Tire { body, contact } => {
                if let Some(body) = bodies.get_mut(*body) {
                    contact.warm_start(body);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, bodies: &mut ObjPool<RigidBody>, dt: f32) {
        match self {
            Self::Tire { body, contact } => {
                if let Some(body) = bodies.get_mut(*body) {
                    contact.solve(body, dt);
                }
            }
        }
    }
}
