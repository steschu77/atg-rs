use crate::v2d::{affine3x3, m3x3::M3x3, v3::V3};
use crate::x2d::constraint::softness::Softness;
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct WheelJoint {
    pub local_anchor_a: V3,
    pub local_anchor_b: V3,
    pub local_axis_b: V3,

    pub rest_length: f32,
    pub softness: Softness,

    accumulated_lambda: [f32; 3],
    effective_mass: [f32; 3],
    bias: [f32; 3],

    n: [V3; 3],

    r_a: V3,
    r_b: V3,

    pub world_anchor_a: V3,
    pub world_anchor_b: V3,

    basis: M3x3,

    pub error: [f32; 3],
}

// ----------------------------------------------------------------------------
impl WheelJoint {
    // ------------------------------------------------------------------------
    pub fn new(
        local_anchor_a: V3,
        local_anchor_b: V3,
        local_axis_b: V3,
        rest_length: f32,
        softness: Softness,
    ) -> Self {
        let basis = affine3x3::basis_from_x0(local_axis_b);

        Self {
            local_anchor_a,
            local_anchor_b,
            local_axis_b: local_axis_b.norm(),

            rest_length,
            softness,

            accumulated_lambda: [0.0; 3],
            effective_mass: [0.0; 3],
            bias: [0.0; 3],

            n: [V3::zero(); 3],

            r_a: V3::zero(),
            r_b: V3::zero(),

            world_anchor_a: V3::zero(),
            world_anchor_b: V3::zero(),

            basis,
            error: [0.0; 3],
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, body_a: &RigidBody, body_b: &RigidBody, dt: f32) {
        self.world_anchor_a = body_a.to_world(self.local_anchor_a);
        self.world_anchor_b = body_b.to_world(self.local_anchor_b);

        self.r_a = self.world_anchor_a - body_a.position();
        self.r_b = self.world_anchor_b - body_b.position();

        let axis = body_b.orientation().rotate(self.basis.col0()).norm();
        let n1 = body_b.orientation().rotate(self.basis.col1()).norm();
        let n2 = body_b.orientation().rotate(self.basis.col2()).norm();

        self.n = [n1, n2, axis];

        let inv_mass_a = body_a.inv_mass();
        let inv_mass_b = body_b.inv_mass();
        let inv_inertia_a = body_a.inv_inertia();
        let inv_inertia_b = body_b.inv_inertia();

        let delta = self.world_anchor_a - self.world_anchor_b;

        for i in 0..3 {
            let rn_a = self.r_a.cross(self.n[i]);
            let rn_b = self.r_b.cross(self.n[i]);

            let k =
                inv_mass_a + inv_mass_b + rn_a * inv_inertia_a * rn_a + rn_b * inv_inertia_b * rn_b;

            self.effective_mass[i] = if k > f32::EPSILON { 1.0 / k } else { 0.0 };

            if i < 2 {
                // slider constraints
                let position_error = self.n[i].dot(delta);
                self.error[i] = position_error;
                self.bias[i] = 0.01 / dt * position_error;
            } else {
                // spring constraint
                let dist = self.n[i].dot(delta);
                let error = dist - self.rest_length;

                self.error[i] = error;
                self.bias[i] = self.softness.bias_rate * error;
            }
        }
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        for i in 0..3 {
            let impulse = self.n[i] * self.accumulated_lambda[i];

            body_a.apply_impulse_at(impulse, self.world_anchor_a, "wheel_warm_start");
            body_b.apply_impulse_at(-impulse, self.world_anchor_b, "wheel_warm_start");
        }
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        let v_a = body_a.velocity_at(self.world_anchor_a);
        let v_b = body_b.velocity_at(self.world_anchor_b);

        for i in 0..3 {
            let c_dot = self.n[i].dot(v_a - v_b);

            let lambda = if i == 2 {
                // suspension softness
                let mass_scale = self.softness.mass_scale;
                let impulse_scale = self.softness.impulse_scale;

                let old_lambda = self.accumulated_lambda[i];
                let lambda = -(c_dot + self.bias[i]) * self.effective_mass[i] * mass_scale
                    - old_lambda * impulse_scale;

                self.accumulated_lambda[i] += lambda;
                self.accumulated_lambda[i] - old_lambda
            } else {
                let lambda = -(c_dot + self.bias[i]) * self.effective_mass[i];
                self.accumulated_lambda[i] += lambda;
                lambda
            };

            let impulse = self.n[i] * lambda;

            body_a.apply_impulse_at(impulse, self.world_anchor_a, "wheel_solve");
            body_b.apply_impulse_at(-impulse, self.world_anchor_b, "wheel_solve");
        }
    }

    // ------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.accumulated_lambda = [0.0; 3];
    }
}
