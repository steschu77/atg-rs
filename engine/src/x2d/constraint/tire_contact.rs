use crate::v2d::{m3x3::M3x3, v3::V3};
use crate::x2d::rigid_body::RigidBody;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct TireContext {
    pub wheel_radius: f32,
    pub contact_point: V3,
    pub world_basis: M3x3,
    pub normal: V3,
    pub penetration: f32,
    pub normal_force: f32,
    pub friction: f32,
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
    lateral_lambda: f32,
    forward_lambda: f32,
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
            lateral_lambda: 0.0,
            forward_lambda: 0.0,
        }
    }

    // ------------------------------------------------------------------------
    pub fn update(&mut self, context: TireContext) {
        self.context = context;
    }

    // ------------------------------------------------------------------------
    pub fn pre_step(&mut self, body: &RigidBody, dt: f32) {
        let inv_mass = body.inv_mass();
        let inv_inertia = body.inv_inertia();

        let normal = self.context.normal;
        let forward = self.context.world_basis.col2();

        let r = self.context.contact_point - body.position();

        let rn_forward = r.cross(forward);
        let rn_normal = r.cross(normal);

        let k_forward = inv_mass + rn_forward * inv_inertia * rn_forward;
        let k_normal = inv_mass + rn_normal * inv_inertia * rn_normal;
        let k_lateral = inv_mass;

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
        let normal = self.context.world_basis.col1();
        body.apply_impulse_at(
            normal * self.normal_lambda,
            self.context.contact_point,
            "tire_normal",
        );

        let lateral = self.context.world_basis.col0();
        body.apply_impulse_at(
            lateral * self.lateral_lambda,
            self.context.contact_point,
            "tire_lateral",
        );

        let forward = self.context.world_basis.col2();
        body.apply_impulse_at(
            forward * self.forward_lambda,
            self.context.contact_point,
            "tire_forward",
        );
    }

    // ------------------------------------------------------------------------
    pub fn solve(&mut self, body: &mut RigidBody, dt: f32) {
        let max_lambda = self.context.friction * self.context.normal_force * dt;

        let v = body.velocity_at(self.context.contact_point);
        let lin_v = body.linear_velocity();

        let lateral = self.context.world_basis.col0();
        let normal = self.context.normal;
        let forward = self.context.world_basis.col2();

        let lateral_speed = lateral.dot(lin_v); // using v would counteract steering impulse
        let forward_speed = forward.dot(v);
        let normal_speed = normal.dot(v);

        let mut lambda = -lateral_speed * self.eff_mass_lateral;
        let old_lambda = self.lateral_lambda;
        self.lateral_lambda = (old_lambda + lambda).clamp(-max_lambda, max_lambda);
        lambda = self.lateral_lambda - old_lambda;

        body.apply_impulse_at(lateral * lambda, self.context.contact_point, "tire_lateral");

        let mut lambda = -forward_speed * self.eff_mass_forward;
        let old_lambda = self.forward_lambda;
        self.forward_lambda = (old_lambda + lambda).clamp(-max_lambda, max_lambda);
        lambda = self.forward_lambda - old_lambda;

        body.apply_impulse_at(forward * lambda, self.context.contact_point, "tire_forward");

        let mut lambda = -(normal_speed + self.bias) * self.eff_mass_normal;
        let old_lambda = self.normal_lambda;
        self.normal_lambda = (old_lambda + lambda).max(0.0);
        lambda = self.normal_lambda - old_lambda;

        body.apply_impulse_at(normal * lambda, self.context.contact_point, "tire_normal");
    }
}
