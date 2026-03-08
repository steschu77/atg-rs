use crate::v2d::v3::V3;
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct DistanceJoint {
    pub local_anchor_a: V3,
    pub local_anchor_b: V3,
    pub rest_length: f32,

    // Solver state
    accumulated_lambda: f32,
    effective_mass: f32,
    bias: f32,

    // Cached per-step data
    n: V3,
    r_a: V3,
    r_b: V3,

    pub world_anchor_a: V3,
    pub world_anchor_b: V3,

    pub error: f32,
}

// ----------------------------------------------------------------------------
impl DistanceJoint {
    // ------------------------------------------------------------------------
    pub fn new(local_anchor_a: V3, local_anchor_b: V3, rest_length: f32) -> Self {
        Self {
            local_anchor_a,
            local_anchor_b,
            rest_length,
            accumulated_lambda: 0.0,
            effective_mass: 0.0,
            bias: 0.0,
            n: V3::zero(),
            r_a: V3::zero(),
            r_b: V3::zero(),
            world_anchor_a: V3::zero(),
            world_anchor_b: V3::zero(),
            error: 0.0,
        }
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, body_a: &RigidBody, body_b: &RigidBody, dt: f32) {
        self.world_anchor_a = body_a.to_world(self.local_anchor_a);
        self.world_anchor_b = body_b.to_world(self.local_anchor_b);

        self.r_a = self.world_anchor_a - body_a.position();
        self.r_b = self.world_anchor_b - body_b.position();

        let delta = self.world_anchor_a - self.world_anchor_b;
        let dist = delta.length();

        if dist > f32::EPSILON {
            self.n = delta / dist;
        } else {
            self.n = V3::zero();
        }

        let inv_mass_a = body_a.inv_mass();
        let inv_mass_b = body_b.inv_mass();
        let inv_inertia_a = body_a.inv_inertia();
        let inv_inertia_b = body_b.inv_inertia();

        let rn_a = self.r_a.cross(self.n);
        let rn_b = self.r_b.cross(self.n);

        let k = inv_mass_a + inv_mass_b + rn_a * inv_inertia_a * rn_a + rn_b * inv_inertia_b * rn_b;

        self.effective_mass = if k > f32::EPSILON { 1.0 / k } else { 0.0 };

        let position_error = dist - self.rest_length;
        self.error = position_error;

        log::info!(
            "pre_step(dt: {dt}) mass_eff: {} → error: {position_error}",
            self.effective_mass,
        );

        self.bias = 0.01 / dt * position_error;
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        let impulse = self.n * self.accumulated_lambda;

        body_a.apply_impulse_at(impulse, self.world_anchor_a, "distance_warm_start");
        body_b.apply_impulse_at(-impulse, self.world_anchor_b, "distance_warm_start");
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, body_a: &mut RigidBody, body_b: &mut RigidBody) {
        let v_a = body_a.velocity_at(self.world_anchor_a);
        let v_b = body_b.velocity_at(self.world_anchor_b);

        let c_dot = self.n.dot(v_a - v_b);

        let lambda = -(c_dot + self.bias) * self.effective_mass;

        self.accumulated_lambda += lambda;

        let impulse = self.n * lambda;

        body_a.apply_impulse_at(impulse, self.world_anchor_a, "distance_solve");
        body_b.apply_impulse_at(-impulse, self.world_anchor_b, "distance_solve");
    }

    // ------------------------------------------------------------------------
    pub fn reset(&mut self) {
        self.accumulated_lambda = 0.0;
    }
}
