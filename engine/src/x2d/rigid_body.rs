use crate::v2d;
use crate::v2d::v2::V2;
use crate::x2d::{Mass, Material};

// ----------------------------------------------------------------------------
pub type RigidBodyId = u32;

// ----------------------------------------------------------------------------
pub struct RigidBody {
    id: RigidBodyId,

    mass: Mass,
    material: Material,

    position: V2,
    velocity: V2,
    force: V2,

    angle: f32,
    angular_velocity: f32,
    torque: f32,
}

impl RigidBody {
    // ------------------------------------------------------------------------
    pub fn new(
        id: RigidBodyId,
        mass: Mass,
        material: Material,
        position: V2,
        velocity: V2,
        force: V2,
        angle: f32,
        angular_velocity: f32,
        torque: f32,
    ) -> Self {
        Self {
            id,
            mass,
            material,
            position,
            velocity,
            force,
            angle,
            angular_velocity,
            torque,
        }
    }

    // ------------------------------------------------------------------------
    pub fn id(&self) -> RigidBodyId {
        self.id
    }

    // ------------------------------------------------------------------------
    pub fn mass(&self) -> f32 {
        self.mass.mass
    }

    // ------------------------------------------------------------------------
    pub fn inv_mass(&self) -> f32 {
        self.mass.inv_mass
    }

    // ------------------------------------------------------------------------
    pub fn inertia(&self) -> f32 {
        self.mass.inertia
    }

    // ------------------------------------------------------------------------
    pub fn inv_inertia(&self) -> f32 {
        self.mass.inv_inertia
    }

    // ------------------------------------------------------------------------
    pub fn pos(&self) -> V2 {
        self.position
    }

    // ------------------------------------------------------------------------
    pub fn vel(&self) -> V2 {
        self.velocity
    }

    // ------------------------------------------------------------------------
    pub fn to_local(&self, world: &V2) -> V2 {
        let r = v2d::r2::R2::new(-self.angle);
        (*world - self.position) * r
    }

    // ------------------------------------------------------------------------
    pub fn to_world(&self, local: &V2) -> V2 {
        let r = v2d::r2::R2::new(self.angle);
        (*local * r) + self.position
    }

    // ------------------------------------------------------------------------
    pub fn velocity_at(&self, world: &V2) -> V2 {
        let r = *world - self.position;
        self.velocity + v2d::v2::V2::s_cross(self.angular_velocity, &r)
    }

    // ------------------------------------------------------------------------
    pub fn apply_force(&mut self, force: &V2) {
        self.force += *force;
    }

    // ------------------------------------------------------------------------
    pub fn apply_impulse(&mut self, impulse: &V2) {
        self.velocity += self.mass.inv_mass * *impulse;
    }

    // ------------------------------------------------------------------------
    pub fn apply_impulse_at(&mut self, impulse: &V2, world: &V2) {
        let r = *world - self.position;
        self.velocity += self.mass.inv_mass * *impulse;
        self.angular_velocity += self.mass.inv_inertia * v2d::v2::V2::cross(&r, impulse);
    }

    // ------------------------------------------------------------------------
    pub fn integrate_forces(&mut self, dt: f32) {
        self.velocity += self.mass.inv_mass * self.force * dt;
        self.angular_velocity += self.mass.inv_inertia * self.torque * dt;

        self.force = V2::zero();
        self.torque = 0.0;
    }

    // ------------------------------------------------------------------------
    pub fn integrate_velocity(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.angle += self.angular_velocity * dt;
    }
}
