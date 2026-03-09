use crate::util::obj_pool::ObjPool;
use crate::v2d::v3::V3;
use crate::x2d::BodyId;
use crate::x2d::constraint::softness::Softness;
use crate::x2d::constraint::{
    distance_joint::DistanceJoint, slider_joint::SliderJoint, spring_joint::SpringJoint,
    wheel_joint::WheelJoint,
};
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Joint {
    Distance {
        body_a: BodyId,
        body_b: BodyId,
        joint: DistanceJoint,
    },
    Slider {
        body_a: BodyId,
        body_b: BodyId,
        joint: SliderJoint,
    },
    Spring {
        body_a: BodyId,
        body_b: BodyId,
        joint: SpringJoint,
    },
    Wheel {
        body_a: BodyId,
        body_b: BodyId,
        joint: WheelJoint,
    },
}

// ----------------------------------------------------------------------------
impl Joint {
    // ------------------------------------------------------------------------
    pub fn new_distance(
        body_a: BodyId,
        body_b: BodyId,
        local_anchor_a: V3,
        local_anchor_b: V3,
        rest_length: f32,
    ) -> Self {
        Self::Distance {
            body_a,
            body_b,
            joint: DistanceJoint::new(local_anchor_a, local_anchor_b, rest_length),
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_spring(
        body_a: BodyId,
        body_b: BodyId,
        local_anchor_a: V3,
        local_anchor_b: V3,
        rest_length: f32,
        softness: Softness,
    ) -> Self {
        Self::Spring {
            body_a,
            body_b,
            joint: SpringJoint::new(local_anchor_a, local_anchor_b, rest_length, softness),
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_slider(
        body_a: BodyId,
        body_b: BodyId,
        local_anchor_a: V3,
        local_anchor_b: V3,
        local_line_dir_b: V3,
    ) -> Self {
        Self::Slider {
            body_a,
            body_b,
            joint: SliderJoint::new(local_anchor_a, local_anchor_b, local_line_dir_b),
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_wheel(
        body_a: BodyId,
        body_b: BodyId,
        local_anchor_a: V3,
        local_anchor_b: V3,
        local_axis_b: V3,
        rest_length: f32,
        softness: Softness,
    ) -> Self {
        Self::Wheel {
            body_a,
            body_b,
            joint: WheelJoint::new(
                local_anchor_a,
                local_anchor_b,
                local_axis_b,
                rest_length,
                softness,
            ),
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, bodies: &mut ObjPool<RigidBody>, dt: f32) {
        match self {
            Self::Distance {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair(*body_a, *body_b) {
                    joint.pre_step(body_a, body_b, dt);
                }
            }

            Self::Spring {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair(*body_a, *body_b) {
                    joint.pre_step(body_a, body_b, dt);
                }
            }

            Self::Slider {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair(*body_a, *body_b) {
                    joint.pre_step(body_a, body_b, dt);
                }
            }

            Self::Wheel {
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
            Self::Distance {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.warm_start(body_a, body_b);
                }
            }

            Self::Spring {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.warm_start(body_a, body_b);
                }
            }

            Self::Slider {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.warm_start(body_a, body_b);
                }
            }

            Self::Wheel {
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
            Self::Distance {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.solve(body_a, body_b);
                }
            }

            Self::Spring {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.solve(body_a, body_b);
                }
            }

            Self::Slider {
                body_a,
                body_b,
                joint,
            } => {
                if let Some((body_a, body_b)) = bodies.get_pair_mut(*body_a, *body_b) {
                    joint.solve(body_a, body_b);
                }
            }

            Self::Wheel {
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
