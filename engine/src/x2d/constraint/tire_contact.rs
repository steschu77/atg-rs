use crate::v2d::{m3x3::M3x3, v3::V3};
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct TireContext {
    pub wheel_radius: f32,
    pub contact_point: V3,
    pub basis: M3x3,
    pub penetration: f32,
    pub normal_force: f32,
    pub friction: f32,
    pub drive_torque: f32,
    pub brake_torque: f32,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct TireContact {
    context: TireContext,

    eff_mass_forward: f32,
    eff_mass_normal: f32,
    eff_mass_lateral: f32,

    bias: f32,
    normal_lambda: f32,
}

// ----------------------------------------------------------------------------
impl TireContact {
    // ------------------------------------------------------------------------
    pub fn new(context: TireContext) -> Self {
        Self {
            context,
            eff_mass_forward: 0.0,
            eff_mass_normal: 0.0,
            eff_mass_lateral: 0.0,
            bias: 0.0,
            normal_lambda: 0.0,
        }
    }

    // ------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, context: TireContext) {
        self.context = context;
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, body: &RigidBody, dt: f32) {
        let inv_mass = body.inv_mass();
        let inv_inertia = body.inv_inertia();

        let lateral = self.context.basis.col0();
        let normal = self.context.basis.col1();
        let forward = self.context.basis.col2();

        let r = self.context.contact_point - body.position();

        let rn_forward = r.cross(forward);
        let rn_normal = r.cross(normal);
        let rn_lateral = r.cross(lateral);

        let k_forward = inv_mass + rn_forward * inv_inertia * rn_forward;
        let k_normal = inv_mass + rn_normal * inv_inertia * rn_normal;
        let k_lateral = inv_mass + rn_lateral * inv_inertia * rn_lateral;

        self.eff_mass_forward = if k_forward > 0.0 {
            1.0 / k_forward
        } else {
            0.0
        };

        self.eff_mass_normal = if k_normal > 0.0 { 1.0 / k_normal } else { 0.0 };

        self.eff_mass_lateral = if k_lateral > 0.0 {
            1.0 / k_lateral
        } else {
            0.0
        };

        self.bias = -(0.1 / dt) * self.context.penetration.max(0.0);
    }

    // ------------------------------------------------------------------------
    pub fn warm_start(&self, body: &mut RigidBody) {
        let normal = self.context.basis.col1();

        body.apply_impulse_at(
            normal * self.normal_lambda,
            self.context.contact_point,
            "tire_normal",
        );
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, body: &mut RigidBody, dt: f32) {
        let max_lambda = self.context.friction * self.context.normal_force * dt;

        let v = body.velocity_at(self.context.contact_point);

        let lateral = self.context.basis.col0();
        let normal = self.context.basis.col1();
        let forward = self.context.basis.col2();

        let lateral_speed = lateral.dot(v);
        let forward_speed = forward.dot(v);

        let mut lambda = -lateral_speed * self.eff_mass_lateral;

        lambda = lambda.clamp(-max_lambda, max_lambda);

        body.apply_impulse_at(lateral * lambda, self.context.contact_point, "tire_lateral");

        let drive_force = self.context.drive_torque / self.context.wheel_radius;
        let brake_force = self.context.brake_torque / self.context.wheel_radius;
        let wheel_force = drive_force - brake_force;

        let mut lambda = wheel_force * dt;

        lambda = lambda.clamp(-max_lambda, max_lambda);

        body.apply_impulse_at(forward * lambda, self.context.contact_point, "tire_forward");

        let c_dot = normal.dot(v);
        let mut lambda = -(c_dot + self.bias) * self.eff_mass_normal;

        let old_lambda = self.normal_lambda;
        self.normal_lambda = (old_lambda + lambda).max(0.0);
        lambda = self.normal_lambda - old_lambda;

        body.apply_impulse_at(
            normal * lambda,
            self.context.contact_point,
            "contact_normal",
        );
    }
}
