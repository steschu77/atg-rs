#![allow(clippy::needless_range_loop)]
use crate::v2d::{m3x3::M3x3, v3::V3};
use crate::x2d::constraint::softness::Softness;
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
const IMPULSE_NAME: [&str; 6] = [
    "wheel_slider_1",
    "wheel_slider_2",
    "wheel_suspension",
    "wheel_ang_motor",
    "wheel_ang_forward",
    "wheel_ang_suspension",
];

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct WheelJoint {
    pub local_anchor_a: V3,
    pub local_anchor_b: V3,
    pub world_basis: M3x3,

    pub rest_length: f32,
    pub softness: Softness,

    pub motor_speed: f32,
    pub max_motor_torque: f32,

    pub accumulated_lambda: [f32; 6],
    pub effective_mass: [f32; 6],
    pub bias: [f32; 6],

    pub n: [V3; 6],

    pub r_a: V3,
    pub r_b: V3,

    pub world_anchor_a: V3,
    pub world_anchor_b: V3,

    pub error: [f32; 6],
}

// ----------------------------------------------------------------------------
impl WheelJoint {
    // ------------------------------------------------------------------------
    pub fn new(
        local_anchor_a: V3,
        local_anchor_b: V3,
        world_basis: M3x3,
        rest_length: f32,
        softness: Softness,
    ) -> Self {
        Self {
            local_anchor_a,
            local_anchor_b,
            world_basis,

            rest_length,
            softness,

            motor_speed: 0.0,
            max_motor_torque: 0.0,

            accumulated_lambda: [0.0; 6],
            effective_mass: [0.0; 6],
            bias: [0.0; 6],

            n: [V3::zero(); 6],

            r_a: V3::zero(),
            r_b: V3::zero(),

            world_anchor_a: V3::zero(),
            world_anchor_b: V3::zero(),

            error: [0.0; 6],
        }
    }

    // ------------------------------------------------------------------------
    pub fn update_motor(&mut self, motor_speed: f32, max_motor_torque: f32) {
        self.motor_speed = motor_speed;
        self.max_motor_torque = max_motor_torque;
    }

    // ------------------------------------------------------------------------
    pub fn update_basis(&mut self, basis: M3x3) {
        self.world_basis = basis;
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, body_a: &RigidBody, body_b: &RigidBody, dt: f32) {
        self.world_anchor_a = body_a.to_world(self.local_anchor_a);
        self.world_anchor_b = body_b.to_world(self.local_anchor_b);

        self.r_a = self.world_anchor_a - body_a.position();
        self.r_b = self.world_anchor_b - body_b.position();

        let w_a = body_a.angular_velocity();
        let w_b = body_b.angular_velocity();

        self.n = [
            self.world_basis.col0(), // lateral
            self.world_basis.col2(), // forward
            self.world_basis.col1(), // suspension
            self.world_basis.col0(), // motor
            self.world_basis.col2(), // forward
            self.world_basis.col1(), // suspension
        ];

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
            } else if i == 2 {
                // spring constraint
                let dist = self.n[i].dot(delta);
                let error = dist - self.rest_length;

                self.error[i] = error;
                self.bias[i] = self.softness.bias_rate * error;
            }
        }

        // angular constraints
        for i in 3..6 {
            let n = self.n[i];
            let k = n * (inv_inertia_a + inv_inertia_b) * n;
            self.effective_mass[i] = if k > f32::EPSILON { 1.0 / k } else { 0.0 };
            self.error[i] = (w_b - w_a).dot(n);
        }
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        for i in 0..3 {
            let impulse = self.n[i] * self.accumulated_lambda[i];

            let info = format!("warm_start_{}", IMPULSE_NAME[i]);
            body_a.apply_impulse_at(impulse, self.world_anchor_a, &info);
            body_b.apply_impulse_at(-impulse, self.world_anchor_b, &info);
        }

        for i in 3..6 {
            let impulse = self.n[i] * self.accumulated_lambda[i];
            let info = format!("warm_start_{}", IMPULSE_NAME[i]);
            body_a.apply_angular_impulse(-impulse, &info);
            body_b.apply_angular_impulse(impulse, &info);
        }
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, body_a: &mut RigidBody, body_b: &mut RigidBody, dt: f32) {
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

            body_a.apply_impulse_at(impulse, self.world_anchor_a, IMPULSE_NAME[i]);
            body_b.apply_impulse_at(-impulse, self.world_anchor_b, IMPULSE_NAME[i]);
        }

        {
            let w_a = body_a.angular_velocity();
            let w_b = body_b.angular_velocity();

            let c_dot = (w_b - w_a).dot(self.n[3]) - self.motor_speed;
            let lambda = -c_dot * self.effective_mass[3];

            let max_lambda = self.max_motor_torque * dt;
            let old_lambda = self.accumulated_lambda[3];
            self.accumulated_lambda[3] = (old_lambda + lambda).clamp(-max_lambda, max_lambda);

            let lambda = self.accumulated_lambda[3] - old_lambda;
            let impulse = self.n[3] * lambda;

            body_a.apply_angular_impulse(-impulse, IMPULSE_NAME[3]);
            body_b.apply_angular_impulse(impulse, IMPULSE_NAME[3]);
        }

        for i in 4..6 {
            let n = self.n[i];

            let w_a = body_a.angular_velocity();
            let w_b = body_b.angular_velocity();

            let c_dot = (w_b - w_a).dot(n);
            let lambda = -c_dot * self.effective_mass[i];

            self.accumulated_lambda[i] += lambda;

            let impulse = n * lambda;

            body_a.apply_angular_impulse(-impulse, IMPULSE_NAME[i]);
            body_b.apply_angular_impulse(impulse, IMPULSE_NAME[i]);
        }
    }

    // ------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.accumulated_lambda = [0.0; 6];
    }
}
