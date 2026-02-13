use crate::core::component::{Component, Context};
use crate::core::game_input::GameKey;
use crate::core::gl_renderer::{RenderContext, RenderObject, Transform};
use crate::error::Result;
use crate::v2d::{affine3x3, m3x3::M3x3, q::Q, v3::V3, v4::V4};
use crate::x2d::{self, mass::Mass, rigid_body::RigidBody};

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct Geometry {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub wheel_base: f32,
    pub wheel_track: f32,
    pub wheel_radius: f32,
    pub wheel_width: f32,
}

// ----------------------------------------------------------------------------
pub const GRAVITY: V3 = V3::new([0.0, -9.81, 0.0]);

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct ChassisData {
    pub steering_angle: f32,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct WheelData {
    pub wheel: WheelPos,
    pub position: V3,
    pub radius: f32,
    pub width: f32,
    pub spin_angle: f32,
    pub rest_length: f32,
    pub spring_k: f32,
    pub damper_c: f32,
    pub compression: f32,
    pub grip: f32,
    pub angular_velocity: f32,
    pub inertia: f32,
    pub drive_torque: f32,
    pub brake_torque: f32,
}

// ----------------------------------------------------------------------------
impl Default for WheelData {
    fn default() -> Self {
        Self {
            wheel: WheelPos::FL,
            position: V3::zero(),
            radius: 0.3,
            width: 0.2,
            spin_angle: 0.0,
            rest_length: 0.8,
            spring_k: 18_000.0,
            damper_c: 2_100.0,
            compression: 0.0,
            grip: 5.0 * 8.0,
            angular_velocity: 0.0,
            inertia: 0.5 * 20.0 * 0.3 * 0.3,
            drive_torque: 0.0,
            brake_torque: 0.0,
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WheelPos {
    FL,
    FR,
    RL,
    RR,
}

// ----------------------------------------------------------------------------
impl From<usize> for WheelPos {
    fn from(value: usize) -> Self {
        match value {
            0 => WheelPos::FL,
            1 => WheelPos::FR,
            2 => WheelPos::RL,
            3 => WheelPos::RR,
            _ => panic!("Invalid wheel index: {}", value),
        }
    }
}
// ----------------------------------------------------------------------------
impl WheelPos {
    pub fn other_lr(self) -> Self {
        match self {
            WheelPos::FL => WheelPos::FR,
            WheelPos::FR => WheelPos::FL,
            WheelPos::RL => WheelPos::RR,
            WheelPos::RR => WheelPos::RL,
        }
    }

    pub fn other_fb(self) -> Self {
        match self {
            WheelPos::FL => WheelPos::RL,
            WheelPos::FR => WheelPos::RR,
            WheelPos::RL => WheelPos::FL,
            WheelPos::RR => WheelPos::FR,
        }
    }

    pub fn index(self) -> usize {
        match self {
            WheelPos::FL => 0,
            WheelPos::FR => 1,
            WheelPos::RL => 2,
            WheelPos::RR => 3,
        }
    }

    pub fn sign_lr(self) -> f32 {
        match self {
            WheelPos::FL | WheelPos::RL => -1.0,
            WheelPos::FR | WheelPos::RR => 1.0,
        }
    }

    pub fn sign_fb(self) -> f32 {
        match self {
            WheelPos::FL | WheelPos::FR => 1.0,
            WheelPos::RL | WheelPos::RR => -1.0,
        }
    }

    pub fn is_front(self) -> bool {
        match self {
            WheelPos::FL | WheelPos::FR => true,
            WheelPos::RL | WheelPos::RR => false,
        }
    }

    pub fn is_rear(self) -> bool {
        match self {
            WheelPos::FL | WheelPos::FR => false,
            WheelPos::RL | WheelPos::RR => true,
        }
    }
}

// ----------------------------------------------------------------------------
fn wheel_basis_static(body: &RigidBody) -> (V3, V3) {
    let forward = body.rotation().rotate(V3::X2);
    let right = body.rotation().rotate(V3::X0);
    (forward, right)
}

// ----------------------------------------------------------------------------
fn wheel_basis_steering(body: &RigidBody, steer_angle: f32) -> (V3, V3) {
    let car_forward = body.rotation().rotate(V3::X2);
    let car_up = body.rotation().rotate(V3::X1);

    let steer_q = Q::from_axis_angle(car_up, steer_angle);
    let forward = steer_q.rotate(car_forward).norm();
    let right = car_up.cross(forward).norm();

    (forward, right)
}

// ----------------------------------------------------------------------------
// Simple raycast for ground plane at y=0. Sophisticated terrain raycasting comes later.
fn raycast_down(origin: V3, max_dist: f32) -> Option<f32> {
    if origin.x1() <= 0.0 {
        return Some(0.0);
    }

    let hit_dist = origin.x1();

    if hit_dist <= max_dist {
        Some(hit_dist)
    } else {
        None
    }
}

// ----------------------------------------------------------------------------
fn apply_wheel_suspension(
    body: &mut RigidBody,
    wheel: &mut WheelData,
    steer_angle: f32,
    brake_strength: f32,
    dt: f32,
) {
    let wheel_pos = body.to_world(wheel.position);
    let ray_len = wheel.rest_length + wheel.radius;

    if let Some(hit_dist) = raycast_down(wheel_pos, ray_len) {
        let compression = wheel.rest_length - (hit_dist - wheel.radius);
        let compression = compression.max(0.0);

        let up = body.rotation().rotate(V3::X1);
        let r = wheel_pos - body.position();
        let v = body.velocity_at(wheel_pos);

        let rel_vel = v.dot(up);

        // Effective mass
        let rn = r.cross(up);
        let inv_mass = body.inv_mass();
        let inv_inertia = body.inv_inertia_tensor;

        let k = inv_mass + rn.dot(inv_inertia * rn);
        if k <= 0.0 {
            return;
        }

        let effective_mass = 1.0 / k;

        // Baumgarte bias (stabilization)
        let beta = 0.5;
        let slop = 0.01;
        let corrected = (compression - slop).max(0.0);
        let bias = beta * corrected / dt;

        // Solve velocity constraint
        let lambda = -(rel_vel + bias) * effective_mass;
        let lambda = lambda.max(0.0);

        body.apply_impulse_at(up * lambda, wheel_pos, "wheel_suspension");
        let normal_impulse = lambda;
        apply_wheel_tire_impulse(
            body,
            wheel,
            wheel_pos,
            normal_impulse,
            steer_angle,
            brake_strength,
        );

        wheel.compression = compression;
    } else {
        wheel.compression = 0.0;
    }
}

// ----------------------------------------------------------------------------
#[allow(clippy::too_many_arguments)]
fn apply_wheel_tire_impulse(
    body: &mut RigidBody,
    wheel: &mut WheelData,
    contact_point: V3,
    normal_impulse: f32,
    steer_angle: f32,
    brake_strength: f32,
) {
    let (forward, right) = if wheel.wheel.is_front() {
        wheel_basis_steering(body, steer_angle)
    } else {
        wheel_basis_static(body)
    };

    let v = body.velocity_at(contact_point);
    let v_forward = v.dot(forward);
    let v_right = v.dot(right);

    let wheel_surface_speed = wheel.angular_velocity * wheel.radius;
    let slip = v_forward - wheel_surface_speed;

    let friction_coeff = 1.2;
    let max_impulse = normal_impulse * friction_coeff;

    let r = contact_point - body.position();
    let r_forward = r.cross(forward);
    let r_right = r.cross(right);

    let inv_mass = body.inv_mass(); // scalar
    let inv_inertia = body.inv_inertia_tensor; // M3x3 (world space)

    let k_right = inv_mass + r_right.dot(inv_inertia * r_right);
    let k_forward = inv_mass + r_forward.dot(inv_inertia * r_forward);

    if k_right > 0.0 {
        let effective_mass = 1.0 / k_right;
        let desired_impulse = -v_right * effective_mass;

        let impulse = desired_impulse.clamp(-max_impulse, max_impulse);
        body.apply_impulse_at(right * impulse, contact_point, "wheel_tire_lateral");
    }

    if k_forward > 0.0 {
        let effective_mass = 1.0 / k_forward;
        let mut desired_impulse = -slip * effective_mass;
        if brake_strength > 0.0 {
            let brake_impulse = -v_forward.signum() * brake_strength * max_impulse;
            desired_impulse += brake_impulse;
        }

        let impulse = desired_impulse.clamp(-max_impulse, max_impulse);
        body.apply_impulse_at(forward * impulse, contact_point, "wheel_tire_longitudinal");

        // Apply opposite torque to wheel
        wheel.angular_velocity += (-impulse * wheel.radius) / wheel.inertia;
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Car {
    pub body: RigidBody,
    pub objects: [RenderObject; 5],
    pub debug_arrows: [RenderObject; 2],
    pub chassis: ChassisData,
    pub wheels: [WheelData; 4],
    pub geometry: Geometry,
    pub engine_force: f32,
    pub brake_force: f32,
}

// ----------------------------------------------------------------------------
impl Car {
    pub fn new(context: &mut RenderContext, geo: Geometry) -> Result<Self> {
        let left_arrow_mesh_id = context.create_colored_mesh(&[], &[], true)?;
        let right_arrow_mesh_id = context.create_colored_mesh(&[], &[], true)?;

        use crate::core::gl_pipeline_colored::{cylinder, transform_mesh};
        let (mut verts, indices) = cylinder(12, geo.wheel_radius, geo.wheel_width);
        transform_mesh(
            &mut verts,
            V3::default(),
            M3x3::from_cols(-V3::X1, V3::X0, V3::X2),
        );
        let wheel_mesh_id = context.create_colored_mesh(&verts, &indices, false)?;
        let dimensions = V3::new([geo.width, 0.2, geo.length]);

        // This is temporary and gives the car 952 kg.
        let material = x2d::WOOD;
        let mass = Mass::from_box(material.density, dimensions)?;
        let body = RigidBody::new(
            mass,
            material,
            V3::new([0.0, 2.0 + geo.wheel_radius + 0.2, 0.0]),
            Q::identity(),
        );

        Ok(Self {
            body,
            objects: [
                RenderObject {
                    name: String::from("car:chassis"),
                    transform: Transform {
                        size: V4::new([geo.width, 0.2, geo.length, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: 0,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:front_left"),
                    transform: Transform {
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:front_right"),
                    transform: Transform {
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:rear_left"),
                    transform: Transform {
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:rear_right"),
                    transform: Transform {
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: 0,
                    ..Default::default()
                },
            ],
            debug_arrows: [
                RenderObject {
                    name: String::from("car:debug_arrow_left"),
                    transform: Transform {
                        position: V4::new([0.0, 0.0, 0.0, 1.0]),
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: left_arrow_mesh_id,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:debug_arrow_right"),
                    transform: Transform {
                        position: V4::new([0.0, 0.0, 0.0, 1.0]),
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: right_arrow_mesh_id,
                    material_id: 0,
                    ..Default::default()
                },
            ],
            chassis: ChassisData {
                ..Default::default()
            },
            wheels: [
                WheelData {
                    wheel: WheelPos::FL,
                    position: V3::new([-0.5 * geo.wheel_track, 0.0, 0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
                WheelData {
                    wheel: WheelPos::FR,
                    position: V3::new([0.5 * geo.wheel_track, 0.0, 0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
                WheelData {
                    wheel: WheelPos::RL,
                    position: V3::new([-0.5 * geo.wheel_track, 0.0, -0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
                WheelData {
                    wheel: WheelPos::RR,
                    position: V3::new([0.5 * geo.wheel_track, 0.0, -0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
            ],
            geometry: geo,
            engine_force: 0.0,
            brake_force: 0.0,
        })
    }

    pub fn position(&self) -> V4 {
        V4::from_v3(self.body.position(), 1.0)
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        use crate::core::gl_pipeline_colored::arrow;

        for i in 0..2 {
            let wheel_pos = self.body.position() + self.wheels[i].position;
            let (forward, _) = wheel_basis_steering(&self.body, self.chassis.steering_angle);
            let forward = V3::new([forward.x0(), 0.0, forward.x2()]);
            let arrow_verts = arrow(wheel_pos, wheel_pos + 1.5 * forward)?;
            context.update_colored_mesh(self.debug_arrows[i].mesh_id, &arrow_verts, &[])?;
        }

        Ok(())
    }

    pub fn transform(&self) -> (V4, V4) {
        let forward = self.body.rotation().rotate(V3::X2);
        let position = self.body.position();
        (V4::from_v3(forward, 0.0), V4::from_v3(position, 1.0))
    }
}

// ----------------------------------------------------------------------------
impl Component for Car {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();

        let engine_torque = if ctx.state.is_pressed(GameKey::Accelerate) {
            1200.0
        } else {
            0.0
        };

        let brake_strength = if ctx.state.is_pressed(GameKey::Brake) {
            1.0
        } else {
            0.0
        };

        if ctx.state.is_pressed(GameKey::SteerLeft) {
            self.chassis.steering_angle -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(GameKey::SteerRight) {
            self.chassis.steering_angle += TURN_SPEED * dt;
        }

        self.body.apply_force(GRAVITY * self.body.mass());
        self.body.integrate_velocities(dt);

        for wheel in &mut self.wheels {
            if !wheel.wheel.is_front() {
                wheel.angular_velocity += (engine_torque / wheel.inertia) * dt;
            }

            if brake_strength > 0.0 {
                let brake_torque = -wheel.angular_velocity * brake_strength * wheel.inertia;
                wheel.angular_velocity += (brake_torque / wheel.inertia) * dt;
            }

            let rolling_drag = 0.02;
            wheel.angular_velocity *= 1.0 - rolling_drag * dt;
        }

        const SOLVER_ITERS: usize = 8;
        for _ in 0..SOLVER_ITERS {
            for wheel in &mut self.wheels {
                apply_wheel_suspension(
                    &mut self.body,
                    wheel,
                    self.chassis.steering_angle,
                    brake_strength,
                    dt,
                );
            }
        }

        Ok(())
    }

    fn integrate_positions(&mut self, dt: f32) {
        self.body.integrate_positions(dt);

        for wheel in &mut self.wheels {
            wheel.spin_angle += wheel.angular_velocity * dt;
        }

        self.objects[0].transform.position = V4::from_v3(self.body.position(), 1.0);
        self.objects[0].transform.rotation = self.body.rotation().into();

        let chassis_rot = self.body.rotation();
        let chassis_transform = self.body.rotation().as_mat3x3();

        for (i, wheel) in &mut self.wheels.iter().enumerate() {
            let steering_angle = if WheelPos::from(i).is_front() {
                -self.chassis.steering_angle
            } else {
                0.0
            };

            let wheel_pos = wheel.position + V3::new([0.0, wheel.compression, 0.0]);
            let wheel_pos = self.body.position() + chassis_transform * wheel_pos;
            let wheel_rot =
                affine3x3::rotate_x1(steering_angle) * affine3x3::rotate_x0(-wheel.spin_angle);
            let wheel_rot = chassis_rot * Q::from_mat3(&wheel_rot);
            self.objects[1 + i].transform.position = V4::from_v3(wheel_pos, 1.0);
            self.objects[1 + i].transform.rotation = wheel_rot.into();
        }
    }
}
