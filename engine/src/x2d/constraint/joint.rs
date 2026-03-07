use crate::util::obj_pool::ObjPool;
use crate::v2d::v3::V3;
use crate::x2d::BodyId;
use crate::x2d::constraint::slider_joint::SliderJoint;
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Joint {
    Slider {
        body_a: BodyId,
        body_b: BodyId,
        joint: SliderJoint,
    },
}

// ----------------------------------------------------------------------------
impl Joint {
    // ------------------------------------------------------------------------
    pub fn new_slider(
        body_a: BodyId,
        body_b: BodyId,
        local_anchor_a: V3,
        local_anchor_b: V3,
        local_line_dir_b: V3,
        beta: f32,
    ) -> Self {
        Self::Slider {
            body_a,
            body_b,
            joint: SliderJoint::new(local_anchor_a, local_anchor_b, local_line_dir_b, beta),
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, bodies: &mut ObjPool<RigidBody>, dt: f32) {
        match self {
            Self::Slider {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair(*body_a, *body_b) {
                    joint.pre_step(body_a, body_b, dt);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, bodies: &mut ObjPool<RigidBody>) {
        match self {
            Self::Slider {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.warm_start(body_a, body_b);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, bodies: &mut ObjPool<RigidBody>) {
        match self {
            Self::Slider {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.solve(body_a, body_b);
                }
            }
        }
    }
}
