use crate::v2d::{affine3x3, m3x3::M3x3, v3::V3};
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct SliderConstraint {
    pub local_anchor_a: V3,
    pub local_anchor_b: V3,
    pub local_line_dir_b: V3,
    pub beta: f32, // Baumgarte stabilization factor

    // Solver state (warm starting)
    accumulated_lambda: [f32; 2],
    effective_mass: [f32; 2],
    bias: [f32; 2],

    // Cached per-step data
    n: [V3; 2],
    r_a: V3,
    r_b: V3,
    pub world_anchor_a: V3,
    pub world_anchor_b: V3,
    basis: M3x3,
}

// ----------------------------------------------------------------------------
impl SliderConstraint {
    // ------------------------------------------------------------------------
    pub fn new(local_anchor_a: V3, local_anchor_b: V3, local_line_dir_b: V3, beta: f32) -> Self {
        let basis = affine3x3::basis_from_x0(local_line_dir_b);
        Self {
            local_anchor_a,
            local_anchor_b,
            local_line_dir_b: local_line_dir_b.norm(),
            beta,
            accumulated_lambda: [0.0; 2],
            effective_mass: [0.0; 2],
            bias: [0.0; 2],
            n: [V3::zero(); 2],
            r_a: V3::zero(),
            r_b: V3::zero(),
            world_anchor_a: V3::zero(),
            world_anchor_b: V3::zero(),
            basis,
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, body_a: &RigidBody, body_b: &RigidBody, _dt: f32) {
        // Compute world anchor
        self.world_anchor_a = body_a.to_world(self.local_anchor_a);
        self.world_anchor_b = body_b.to_world(self.local_anchor_b);

        self.r_a = self.world_anchor_a - body_a.position();
        self.r_b = self.world_anchor_b - body_b.position();

        // update the perpendicular basis
        let n1 = body_b.rotation().rotate(self.basis.col1()).norm();
        let n2 = body_b.rotation().rotate(self.basis.col2()).norm();

        self.n = [n1, n2];

        let inv_mass_a = body_a.inv_mass();
        let inv_mass_b = body_b.inv_mass();
        let inv_inertia_a = body_a.inv_inertia();
        let inv_inertia_b = body_b.inv_inertia();

        for i in 0..2 {
            let rn_a = self.r_a.cross(self.n[i]);
            let rn_b = self.r_b.cross(self.n[i]);

            let k =
                inv_mass_a + inv_mass_b + rn_a * inv_inertia_a * rn_a + rn_b * inv_inertia_b * rn_b;

            self.effective_mass[i] = if k > f32::EPSILON { 1.0 / k } else { 0.0 };

            let position_error = self.n[i].dot(self.world_anchor_a - self.world_anchor_b);
            log::info!(
                "pre_step: position_error[{}] = {}, k = {}",
                i,
                position_error,
                k
            );
            //self.bias[i] = self.beta / dt * position_error;
        }
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        for i in 0..2 {
            let impulse = self.n[i] * self.accumulated_lambda[i];
            body_a.apply_impulse_at(impulse, self.world_anchor_a, "slider_warm_start");
            body_b.apply_impulse_at(-impulse, self.world_anchor_b, "slider_warm_start");
        }
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        for i in 0..2 {
            let v_a = body_a.velocity_at(self.world_anchor_a);
            let v_b = body_b.velocity_at(self.world_anchor_b);

            let c_dot = self.n[i].dot(v_a - v_b);

            let lambda = -(c_dot + self.bias[i]) * self.effective_mass[i];

            self.accumulated_lambda[i] += lambda;
            let impulse = self.n[i] * lambda;

            body_a.apply_impulse_at(impulse, self.world_anchor_a, "slider_solve");
            body_b.apply_impulse_at(-impulse, self.world_anchor_b, "slider_solve");
        }
    }

    // ------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.accumulated_lambda = [0.0; 2];
    }
}
