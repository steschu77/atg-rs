use crate::core::gl_renderer::Transform;
use crate::util::obj_pool::ObjPool;
use crate::x2d::{BodyId, JointId, constraint::joint::Joint, rigid_body::RigidBody};

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct World {
    bodies: ObjPool<RigidBody>,
    joints: ObjPool<Joint>,
}

// ----------------------------------------------------------------------------
impl Default for World {
    fn default() -> Self {
        Self {
            bodies: ObjPool::new(),
            joints: ObjPool::new(),
        }
    }
}

// ----------------------------------------------------------------------------
impl World {
    // ------------------------------------------------------------------------
    pub fn new() -> Self {
        Self::default()
    }

    // ------------------------------------------------------------------------
    pub fn add_body(&mut self, body: RigidBody) -> BodyId {
        self.bodies.insert(body)
    }

    // ------------------------------------------------------------------------
    pub fn remove_body(&mut self, id: BodyId) {
        self.bodies.remove(id);
    }

    // ------------------------------------------------------------------------
    pub fn update_body(&self, id: BodyId, transform: &mut Transform) {
        if let Some(body) = self.bodies.get(id) {
            *transform = body.transform();
        }
    }

    // ------------------------------------------------------------------------
    pub fn get_body_mut(&mut self, id: BodyId) -> Option<&mut RigidBody> {
        self.bodies.get_mut(id)
    }

    // ------------------------------------------------------------------------
    pub fn add_joint(&mut self, joint: Joint) -> JointId {
        self.joints.insert(joint)
    }

    // ------------------------------------------------------------------------
    pub fn remove_joint(&mut self, id: JointId) {
        self.joints.remove(id);
    }

    // ------------------------------------------------------------------------
    pub fn step(&mut self, dt: f32) {
        self.integrate_forces(dt);
        self.pre_step(dt);
        self.warm_start();

        for _ in 0..10 {
            self.solve_constraints();
        }

        self.integrate_velocities(dt);
    }

    // ------------------------------------------------------------------------
    fn integrate_forces(&mut self, dt: f32) {
        for body in self.bodies.iter_mut() {
            body.integrate_forces(dt);
        }
    }

    // ------------------------------------------------------------------------
    fn pre_step(&mut self, dt: f32) {
        for joint in self.joints.iter_mut() {
            joint.pre_step(&mut self.bodies, dt);
        }
    }

    // ------------------------------------------------------------------------
    fn warm_start(&mut self) {
        for joint in self.joints.iter() {
            joint.warm_start(&mut self.bodies);
        }
    }

    // ------------------------------------------------------------------------
    fn solve_constraints(&mut self) {
        for joint in self.joints.iter_mut() {
            joint.solve(&mut self.bodies);
        }
    }

    // ------------------------------------------------------------------------
    fn integrate_velocities(&mut self, dt: f32) {
        for body in self.bodies.iter_mut() {
            body.integrate_velocities(dt);
        }
    }
}
