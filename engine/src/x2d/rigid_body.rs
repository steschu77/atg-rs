use crate::v2d::{m3x3::M3x3, q::Q, v3::V3};
use crate::x2d::{Material, mass::Mass};

// ----------------------------------------------------------------------------
// This file implements a simple sphere rigid body. The physics is based on the
// "semi-implicit Euler" method, which is a simple and stable integration scheme
// for rigid body dynamics.
//
// Online resources:
// https://gafferongames.com/post/physics_in_3d/
// https://www.cs.cmu.edu/~baraff/sigcourse/notesd1.pdf

// ----------------------------------------------------------------------------
pub fn from_angular_velocity(omega_dt: V3) -> Q {
    let angle = omega_dt.length();
    if angle < 1.0e-6 {
        // Very small rotation → identity
        Q::identity()
    } else {
        let axis = omega_dt * (1.0 / angle);
        Q::from_axis_angle(axis, angle)
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct RigidBody {
    mass: Mass,
    material: Material,

    pos: V3,
    rot: Q,

    linear_vel: V3,
    angular_vel: V3,

    force: V3,
    torque: V3,
}

// ----------------------------------------------------------------------------
impl RigidBody {
    // ------------------------------------------------------------------------
    pub fn new(mass: Mass, material: Material, pos: V3, rot: Q) -> Self {
        Self {
            mass,
            material,
            pos,
            rot,
            linear_vel: V3::zero(),
            angular_vel: V3::zero(),
            force: V3::zero(),
            torque: V3::zero(),
        }
    }

    // ------------------------------------------------------------------------
    pub fn mass(&self) -> f32 {
        self.mass.mass()
    }

    // ------------------------------------------------------------------------
    pub fn inv_mass(&self) -> f32 {
        self.mass.inv_mass()
    }

    // ------------------------------------------------------------------------
    pub fn inertia(&self) -> V3 {
        self.mass.inertia()
    }

    // ------------------------------------------------------------------------
    pub fn inv_inertia(&self) -> V3 {
        self.mass.inv_inertia()
    }

    // ------------------------------------------------------------------------
    pub fn position(&self) -> V3 {
        self.pos
    }

    // ------------------------------------------------------------------------
    pub fn velocity(&self) -> V3 {
        self.linear_vel
    }

    // ------------------------------------------------------------------------
    pub fn to_local(&self, world: V3) -> V3 {
        let r = world - self.pos;
        self.rot.inv_rotate(r)
    }

    // ------------------------------------------------------------------------
    pub fn to_world(&self, local: V3) -> V3 {
        self.rot.rotate(local) + self.pos
    }

    // ------------------------------------------------------------------------
    pub fn velocity_at(&self, world_pt: V3) -> V3 {
        let r = world_pt - self.pos;
        self.linear_vel + self.angular_vel.cross(r)
    }

    // ------------------------------------------------------------------------
    pub fn apply_force(&mut self, force: V3) {
        self.force += force;
    }

    // ------------------------------------------------------------------------
    pub fn apply_force_at(&mut self, force: V3, world_pt: V3) {
        self.force += force;

        let r = world_pt - self.pos;
        self.torque += r.cross(force);
    }

    // ------------------------------------------------------------------------
    pub fn integrate(&mut self, dt: f32) {
        // Apply and clear accumulators
        let lin_accel = self.force * self.inv_mass();

        // This ignores gyroscopic terms (ω × Iω) for stability and simplicity.
        let rot_mat = self.rot.as_mat3x3();
        let inv_inertia_world = rot_mat * M3x3::diag(self.inv_inertia()) * rot_mat.transpose();
        let ang_accel = inv_inertia_world * self.torque;

        self.force = V3::zero();
        self.torque = V3::zero();

        self.linear_vel += lin_accel * dt;
        self.angular_vel += ang_accel * dt;

        self.pos += self.linear_vel * dt;

        let dq = from_angular_velocity(self.angular_vel * dt);
        self.rot = (self.rot * dq).norm();
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_float_eq;

    use super::*;

    #[test]
    fn rigid_body_no_force_no_move() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        body.integrate(1.0);

        assert_eq!(body.position(), V3::zero());
        assert_eq!(body.velocity(), V3::zero());
        assert_eq!(body.angular_vel, V3::zero());
    }

    #[test]
    fn rigid_body_constant_force_accelerates_linearly() {
        let mut body = RigidBody::new(
            Mass::new(2.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        let force = V3::new([4.0, 0.0, 0.0]); // a = 2
        body.apply_force_at(force, V3::zero());

        body.integrate(1.0);
        assert_eq!(body.velocity(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.position(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.angular_vel, V3::zero());

        // accumulators should be cleared, so no more acceleration
        body.integrate(1.0);
        assert_eq!(body.velocity(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.position(), V3::new([4.0, 0.0, 0.0]));
        assert_eq!(body.angular_vel, V3::zero());
    }

    #[test]
    fn test_rigid_body() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        // Move and rotate upwards around Z axis
        body.apply_force_at(V3::new([0.0, 1.0, 0.0]), V3::new([1.0, 0.0, 0.0]));

        body.integrate(1.0);
        assert!(body.position().x1() > 0.0);
        assert!(body.angular_vel.x2() > 0.0);
    }

    #[test]
    fn to_local_to_world_identity() {
        let body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        let p = V3::new([1.0, 2.0, 3.0]);

        assert_eq!(body.to_local(p), p);
        assert_eq!(body.to_world(p), p);
    }

    #[test]
    fn to_local_to_world_translation_only() {
        let body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::new([10.0, 0.0, 0.0]),
            Q::identity(),
        );

        let world = V3::new([11.0, 2.0, -3.0]);
        let local = V3::new([1.0, 2.0, -3.0]);

        assert_eq!(body.to_local(world), local);
        assert_eq!(body.to_world(local), world);
    }

    #[test]
    fn to_local_to_world_rotation_only() {
        let rot = Q::from_axis_angle(V3::X2, std::f32::consts::FRAC_PI_2);

        let body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            rot,
        );

        let local = V3::new([1.0, 0.0, 0.0]);
        let world = V3::new([0.0, 1.0, 0.0]);

        assert_eq!(body.to_world(local), world);
        assert_eq!(body.to_local(world), local);
    }

    #[test]
    fn to_local_to_world_round_trip() {
        let rot = Q::from_axis_angle(V3::X2, 0.7);

        let body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::new([3.0, -2.0, 5.0]),
            rot,
        );

        let world = V3::new([-4.0, 1.5, 2.0]);
        let local = body.to_local(world);
        let world2 = body.to_world(local);

        assert_eq!(world, world2);
    }

    #[test]
    fn angular_velocity_world_space_rotation_direction() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        // Angular velocity: +90°/s around Z
        body.angular_vel = V3::new([0.0, 0.0, std::f32::consts::FRAC_PI_2]);

        body.integrate(1.0);

        // Rotate local X axis into world space
        let x_world = body.to_world(V3::X0);

        assert_float_eq!(x_world.x0(), 0.0);
        assert_float_eq!(x_world.x1(), 1.0);
    }
}
