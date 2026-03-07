use crate::core::gl_renderer::Transform;
use crate::v2d::{m3x3::M3x3, q::Q, v3::V3, v4::V4};
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
    let angle2 = omega_dt.length2();
    if angle2 < 1.0e-12 {
        Q::new([
            0.5 * omega_dt.x0(),
            0.5 * omega_dt.x1(),
            0.5 * omega_dt.x2(),
            1.0,
        ])
        .norm()
    } else {
        let angle = angle2.sqrt();
        let axis = omega_dt * (1.0 / angle);
        Q::from_axis_angle(axis, angle)
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct RigidBody {
    mass: Mass,
    material: Material,

    position: V3,
    orientation: Q,

    linear_vel: V3,
    angular_vel: V3,

    force_accu: V3,
    torque_accu: V3,

    inv_inertia_world: M3x3,
}

// ----------------------------------------------------------------------------
impl RigidBody {
    // ------------------------------------------------------------------------
    pub fn new(mass: Mass, material: Material, pos: V3, rot: Q) -> Self {
        Self {
            mass,
            material,
            position: pos,
            orientation: rot,
            linear_vel: V3::zero(),
            angular_vel: V3::zero(),
            force_accu: V3::zero(),
            torque_accu: V3::zero(),
            inv_inertia_world: Self::update_inertia_world(rot, mass.inv_inertia()),
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
    pub fn inv_inertia(&self) -> M3x3 {
        self.inv_inertia_world
    }

    // ------------------------------------------------------------------------
    pub fn restitution(&self) -> f32 {
        self.material.restitution
    }

    // ------------------------------------------------------------------------
    pub fn friction(&self) -> f32 {
        self.material.static_friction
    }

    // ------------------------------------------------------------------------
    pub fn position(&self) -> V3 {
        self.position
    }

    // ------------------------------------------------------------------------
    pub fn orientation(&self) -> Q {
        self.orientation
    }

    // ------------------------------------------------------------------------
    pub fn linear_velocity(&self) -> V3 {
        self.linear_vel
    }

    // ------------------------------------------------------------------------
    pub fn angular_velocity(&self) -> V3 {
        self.angular_vel
    }

    // ------------------------------------------------------------------------
    pub fn to_local(&self, world: V3) -> V3 {
        let r = world - self.position;
        self.orientation.inv_rotate(r)
    }

    // ------------------------------------------------------------------------
    pub fn to_world(&self, local: V3) -> V3 {
        self.orientation.rotate(local) + self.position
    }

    // ------------------------------------------------------------------------
    pub fn velocity_at(&self, world_pt: V3) -> V3 {
        let r = world_pt - self.position;
        self.linear_vel + self.angular_vel.cross(r)
    }

    // ------------------------------------------------------------------------
    pub fn apply_force(&mut self, force: V3) {
        log::info!("RigidBody::apply_force(force: {force})");
        self.force_accu += force;
    }

    // ------------------------------------------------------------------------
    pub fn apply_force_at(&mut self, force: V3, world_pt: V3) {
        log::info!("RigidBody::apply_force_at(force: {force}, world_pt: {world_pt})");
        self.force_accu += force;

        let r = world_pt - self.position;
        self.torque_accu += r.cross(force);
    }

    // ------------------------------------------------------------------------
    pub fn apply_impulse(&mut self, impulse: V3, reason: &str) {
        log::info!("RigidBody::impulse[{reason}](impulse: {impulse})");
        self.linear_vel += impulse * self.inv_mass();
    }

    // ------------------------------------------------------------------------
    pub fn apply_impulse_at(&mut self, impulse: V3, world_pt: V3, reason: &str) {
        log::info!("RigidBody::impulse[{reason}](impulse: {impulse}, pt: {world_pt})");

        // Linear velocity
        self.linear_vel += impulse * self.inv_mass();

        // Angular velocity
        let r = world_pt - self.position;
        let angular_impulse = r.cross(impulse);

        self.angular_vel += self.inv_inertia_world * angular_impulse;
    }

    // ------------------------------------------------------------------------
    pub fn apply_angular_impulse(&mut self, impulse: V3, reason: &str) {
        log::info!("RigidBody::angular_impulse[{reason}](impulse: {impulse})");
        self.angular_vel += self.inv_inertia() * impulse;
    }

    // ------------------------------------------------------------------------
    pub fn integrate_forces(&mut self, dt: f32) {
        let lin_accel = self.force_accu * self.inv_mass();
        let ang_accel = self.inv_inertia_world * self.torque_accu;

        self.linear_vel += lin_accel * dt;
        self.angular_vel += ang_accel * dt;

        log::info!(
            "RigidBody::integrate_forces(dt: {dt}) → force: {}, torque: {}, linear_vel: {}, angular_vel: {}",
            self.force_accu,
            self.torque_accu,
            self.linear_vel,
            self.angular_vel,
        );

        self.force_accu = V3::zero();
        self.torque_accu = V3::zero();
    }

    // ------------------------------------------------------------------------
    pub fn integrate_velocities(&mut self, dt: f32) {
        self.position += self.linear_vel * dt;

        let dq = from_angular_velocity(self.angular_vel * dt);
        self.orientation = (dq * self.orientation).norm();

        self.inv_inertia_world =
            Self::update_inertia_world(self.orientation, self.mass.inv_inertia());

        log::info!(
            "RigidBody::integrate_vel(dt: {dt}) → pos: {}, rot: {}",
            self.position,
            self.orientation,
        );
    }

    // ------------------------------------------------------------------------
    pub fn angular_momentum(&self) -> V3 {
        self.inv_inertia_world.inverse() * self.angular_vel
    }

    // ------------------------------------------------------------------------
    pub fn kinetic_energy(&self) -> f32 {
        let linear = 0.5 * self.mass() * self.linear_vel.length2();

        let intertia = self.inv_inertia_world.inverse();
        let rotational = 0.5 * self.angular_vel.dot(intertia * self.angular_vel);

        linear + rotational
    }

    // ------------------------------------------------------------------------
    pub fn transform(&self) -> Transform {
        Transform {
            position: V4::from_v3(self.position, 1.0),
            rotation: self.orientation.into(),
            ..Default::default()
        }
    }

    // ------------------------------------------------------------------------
    pub fn log(&self) {
        log::info!("RigidBody: {self:?}");
    }

    // ------------------------------------------------------------------------
    fn update_inertia_world(orientation: Q, inv_inertia_body: V3) -> M3x3 {
        let r = orientation.as_mat3x3();
        r * M3x3::diag(inv_inertia_body) * r.transpose()
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

        body.integrate_velocities(1.0);

        assert_eq!(body.position(), V3::zero());
        assert_eq!(body.linear_velocity(), V3::zero());
        assert_eq!(body.angular_velocity(), V3::zero());
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

        body.integrate_forces(1.0);
        body.integrate_velocities(1.0);
        assert_eq!(body.linear_velocity(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.position(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.angular_velocity(), V3::zero());

        // accumulators should be cleared, so no more acceleration
        body.integrate_forces(1.0);
        body.integrate_velocities(1.0);
        assert_eq!(body.linear_velocity(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.position(), V3::new([4.0, 0.0, 0.0]));
        assert_eq!(body.angular_velocity(), V3::zero());
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

        body.integrate_forces(1.0);
        body.integrate_velocities(1.0);
        assert!(body.position().x1() > 0.0);
        assert!(body.angular_velocity().x2() > 0.0);
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

        body.integrate_forces(1.0);
        body.integrate_velocities(1.0);

        // Rotate local X axis into world space
        let x_world = body.to_world(V3::X0);

        assert_float_eq!(x_world.x0(), 0.0);
        assert_float_eq!(x_world.x1(), 1.0);
    }

    // Linear impulse at center → linear velocity only
    // This verifies:
    // * impulse changes velocity immediately
    // * position does not change until integration
    // * no angular velocity is introduced
    #[test]
    fn apply_impulse_linear_only() {
        let mut body = RigidBody::new(
            Mass::new(2.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        let impulse = V3::new([4.0, 0.0, 0.0]); // Δv = 2
        body.apply_impulse(impulse, "test");

        assert_eq!(body.position(), V3::zero());
        assert_eq!(body.linear_velocity(), V3::new([2.0, 0.0, 0.0]));
        assert_eq!(body.angular_velocity(), V3::zero());

        body.integrate_forces(1.0);
        body.integrate_velocities(1.0);
        assert_eq!(body.position(), V3::new([2.0, 0.0, 0.0]));
    }

    // This test verifies that applying an impulse at an offset generates the expected angular velocity.
    // This verifies:
    // * angular impulse = r × J
    // * correct rotation axis
    // * no unwanted coupling
    #[test]
    fn apply_impulse_at_generates_angular_velocity() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        // Impulse in +Y at +X → torque around +Z
        body.apply_impulse_at(V3::new([0.0, 1.0, 0.0]), V3::new([1.0, 0.0, 0.0]), "test");

        assert_eq!(body.linear_velocity(), V3::new([0.0, 1.0, 0.0]));
        assert!(body.angular_velocity().x2() > 0.0);
        assert_float_eq!(body.angular_velocity().x0(), 0.0);
        assert_float_eq!(body.angular_velocity().x1(), 0.0);
    }

    #[test]
    // If this fails → cross product or inertia is wrong.
    fn equal_opposite_impulses_pure_rotation() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        let r = V3::new([1.0, 0.0, 0.0]);
        let j = V3::new([0.0, 1.0, 0.0]);

        body.apply_impulse_at(j, r, "test");
        body.apply_impulse_at(-j, -r, "test");

        // Linear must cancel
        assert_float_eq!(body.linear_velocity().length(), 0.0);

        // Angular must be non-zero
        assert!(body.angular_velocity().length() > 0.0);
    }

    #[test]
    // No forces → momentum must stay constant.
    // If this drifts → integration is wrong.
    fn linear_momentum_conserved() {
        let mut body = RigidBody::new(
            Mass::new(2.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        body.apply_impulse(V3::new([4.0, 3.0, 2.0]), "test");

        let initial = body.linear_velocity();

        for _ in 0..1000 {
            body.integrate_forces(0.01);
            body.integrate_velocities(0.01);
        }

        assert_float_eq!(body.linear_velocity().x0(), initial.x0());
        assert_float_eq!(body.linear_velocity().x1(), initial.x1());
        assert_float_eq!(body.linear_velocity().x2(), initial.x2());
    }

    #[test]
    // If symmetry breaks → matrix multiplication issue.
    fn inertia_tensor_stays_symmetric() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::new([2.0, 3.0, 4.0])).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        body.angular_vel = V3::new([1.0, 2.0, 3.0]);

        for _ in 0..1000 {
            body.integrate_velocities(0.01);
        }

        let inv_inertia = body.inv_inertia();
        assert_float_eq!(inv_inertia.x01(), inv_inertia.x10());
        assert_float_eq!(inv_inertia.x02(), inv_inertia.x20());
        assert_float_eq!(inv_inertia.x12(), inv_inertia.x21());
    }

    #[test]
    fn asymmetric_body_free_spin_conserves_angular_momentum() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::new([2.0, 2.1, 2.0])).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        body.apply_angular_impulse(V3::new([0.3, 0.7, 1.1]), "test");

        let initial_angular_momentum = body.angular_momentum();

        for _ in 0..5000 {
            body.integrate_velocities(0.001);
        }

        let final_angular_momentum = body.angular_momentum();
        let diff = (initial_angular_momentum - final_angular_momentum).length();
        assert!(diff < 1e-1, "Angular momentum not conserved: diff = {diff}");
    }

    #[test]
    fn conserve_kinetic_energy() {
        let mut body = RigidBody::new(
            Mass::new(2.0, V3::one()).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        body.apply_angular_impulse(V3::new([0.3, 0.7, 1.1]), "test");
        body.apply_impulse(V3::new([4.0, 0.0, 0.0]), "test");

        let initial = body.kinetic_energy();

        for _ in 0..10_000 {
            body.integrate_velocities(0.001);
        }

        let final_energy = body.kinetic_energy();

        assert!((final_energy - initial).abs() < 1e-6);
    }

    #[test]
    fn stress_free_spin_stability() {
        let mut body = RigidBody::new(
            Mass::new(1.0, V3::new([2.0, 3.0, 4.0])).unwrap(),
            Material::default(),
            V3::zero(),
            Q::identity(),
        );

        body.angular_vel = V3::new([1.3, -2.1, 0.7]);

        let dt = 0.001;
        let steps = 200_000;

        let mut max_omega = 0.0f32;
        let mut max_q_error = 0.0f32;

        for _ in 0..steps {
            body.integrate_velocities(dt);

            // track angular velocity growth
            let omega_len = body.angular_velocity().length();
            max_omega = max_omega.max(omega_len);

            // track quaternion normalization error
            let q_len = body.orientation().length();
            max_q_error = max_q_error.max((q_len - 1.0).abs());

            assert!(omega_len.is_finite());
            assert!(q_len.is_finite());
        }

        // Angular velocity should stay bounded
        assert!(max_omega < 10.0);

        // Quaternion should remain normalized
        assert!(max_q_error < 1e-5);
    }
}
