use crate::core::gl_renderer::Transform;
use crate::util::obj_pool::ObjPool;
use crate::x2d::{
    BodyId, ContactId, JointId, constraint::contact::Contact, constraint::joint::Joint,
    rigid_body::RigidBody,
};

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Physics {
    bodies: ObjPool<RigidBody>,
    joints: ObjPool<Joint>,
    contacts: ObjPool<Contact>,
}

// ----------------------------------------------------------------------------
impl Default for Physics {
    fn default() -> Self {
        Self {
            bodies: ObjPool::new(),
            joints: ObjPool::new(),
            contacts: ObjPool::new(),
        }
    }
}

// ----------------------------------------------------------------------------
impl Physics {
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
    pub fn get_body(&self, id: BodyId) -> Option<&RigidBody> {
        self.bodies.get(id)
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
    pub fn get_joint(&self, id: JointId) -> Option<&Joint> {
        self.joints.get(id)
    }

    // ------------------------------------------------------------------------
    pub fn get_joint_mut(&mut self, id: JointId) -> Option<&mut Joint> {
        self.joints.get_mut(id)
    }

    // ------------------------------------------------------------------------
    pub fn add_contact(&mut self, contact: Contact) -> ContactId {
        self.contacts.insert(contact)
    }

    // ------------------------------------------------------------------------
    pub fn remove_contact(&mut self, id: ContactId) {
        self.contacts.remove(id);
    }

    // ------------------------------------------------------------------------
    pub fn get_contact(&self, id: ContactId) -> Option<&Contact> {
        self.contacts.get(id)
    }

    // ------------------------------------------------------------------------
    pub fn get_contact_mut(&mut self, id: ContactId) -> Option<&mut Contact> {
        self.contacts.get_mut(id)
    }

    // ------------------------------------------------------------------------
    pub fn step(&mut self, dt: f32) {
        self.integrate_forces(dt);
        self.pre_step(dt);
        self.warm_start();

        let solver_iterations = 10;
        //let dt_solver = dt / solver_iterations as f32;
        let dt_solver = dt;
        for _ in 0..solver_iterations {
            self.solve_contacts(dt_solver);
            self.solve_constraints(dt_solver);
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
        for contact in self.contacts.iter_mut() {
            contact.pre_step(&mut self.bodies, dt);
        }
    }

    // ------------------------------------------------------------------------
    fn warm_start(&mut self) {
        for joint in self.joints.iter() {
            joint.warm_start(&mut self.bodies);
        }
        for contact in self.contacts.iter() {
            contact.warm_start(&mut self.bodies);
        }
    }

    // ------------------------------------------------------------------------
    fn solve_constraints(&mut self, dt: f32) {
        for joint in self.joints.iter_mut() {
            joint.solve(&mut self.bodies, dt);
        }
    }

    // ------------------------------------------------------------------------
    fn solve_contacts(&mut self, dt: f32) {
        for contact in self.contacts.iter_mut() {
            contact.solve(&mut self.bodies, dt);
        }
    }

    // ------------------------------------------------------------------------
    fn integrate_velocities(&mut self, dt: f32) {
        for body in self.bodies.iter_mut() {
            body.integrate_velocities(dt);
        }
    }
}
