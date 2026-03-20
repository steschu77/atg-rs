use crate::core::component::Context;
use crate::core::game_input::GameKey;
use crate::core::gl_pipeline_colored::arrow;
use crate::core::gl_renderer::{
    DefaultMaterials, DefaultMeshes, RenderContext, RenderObject, Transform,
};
use crate::core::terrain::Terrain;
use crate::error::{Error, Result};
use crate::v2d::{m3x3::M3x3, q::Q, v3::V3, v4::V4};
use crate::x2d::{
    self, BodyId, ContactId, JointId, constraint::contact::Contact, constraint::joint::Joint,
    constraint::softness::Softness, constraint::tire_contact::TireContext, mass::Mass,
    physics::Physics, rigid_body::RigidBody,
};
use std::fmt;

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
#[derive(Debug, Clone)]
pub struct WheelData {
    pub is_steering: bool,
    pub is_driving: bool,
    pub local_position: V3,
    pub radius: f32,
    pub width: f32,
    pub body: BodyId,
    pub joint: JointId,
    pub contact: Option<ContactId>,
}

// ----------------------------------------------------------------------------
impl WheelData {
    fn new(
        is_steering: bool,
        is_driving: bool,
        local_position: V3,
        body: BodyId,
        wheel_joint: JointId,
        radius: f32,
        width: f32,
    ) -> Self {
        Self {
            is_steering,
            is_driving,
            local_position,
            radius,
            width,
            body,
            joint: wheel_joint,
            contact: None,
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DriveDirection {
    #[default]
    Forward,
    Reverse,
}

// ----------------------------------------------------------------------------
impl fmt::Display for DriveDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriveDirection::Forward => write!(f, "Forward"),
            DriveDirection::Reverse => write!(f, "Reverse"),
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DriveState {
    #[default]
    Coast,
    Drive,
    DriveBraking,
    Braking,
    Stopped,
}

// ----------------------------------------------------------------------------
impl fmt::Display for DriveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriveState::Coast => write!(f, "Coast"),
            DriveState::Drive => write!(f, "Drive"),
            DriveState::DriveBraking => write!(f, "DriveBraking"),
            DriveState::Braking => write!(f, "Braking"),
            DriveState::Stopped => write!(f, "Stopped"),
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct DriveStateContext {
    pub state: DriveState,
    pub direction: DriveDirection,
    pub state_time: f32,
}

const STOP_DELAY: f32 = 0.3;
const V_BACKWARD: f32 = -0.5;
const V_EPSILON: f32 = 0.1;

// ----------------------------------------------------------------------------
// `drive` and `resist` are abstract — caller maps throttle/brake to them
fn update_drive_state(state: DriveState, drive: bool, brake: bool, near_stop: bool) -> DriveState {
    debug_assert!(
        !matches!(state, DriveState::Coast | DriveState::Stopped),
        "Coast and Stopped are handled by update_direction_state"
    );
    match (drive, brake) {
        (false, false) => DriveState::Coast,
        (true, false) => DriveState::Drive,
        (false, true) => match state {
            DriveState::Drive => DriveState::Coast,
            DriveState::DriveBraking if near_stop => DriveState::Stopped,
            DriveState::DriveBraking => DriveState::Braking,
            DriveState::Braking if near_stop => DriveState::Stopped,
            DriveState::Braking => DriveState::Braking,
            DriveState::Stopped => DriveState::Stopped,
            DriveState::Coast => DriveState::Braking,
        },
        (true, true) => DriveState::DriveBraking,
    }
}

// ----------------------------------------------------------------------------
fn update_direction_state(
    ctx: &DriveStateContext,
    throttle: bool,
    brake: bool,
    v_long: f32,
    dt: f32,
) -> DriveStateContext {
    let state_time = ctx.state_time + dt;
    let near_stop = v_long.abs() < V_EPSILON;

    match ctx.state {
        DriveState::Coast => {
            let going_forward = v_long >= V_BACKWARD;
            let (direction, drive, resist) = if going_forward {
                (DriveDirection::Forward, throttle, brake)
            } else {
                (DriveDirection::Reverse, brake, throttle)
            };
            let next = match (drive, resist) {
                (false, false) => DriveState::Coast,
                (true, false) => DriveState::Drive,
                (false, true) => DriveState::Braking,
                (true, true) => DriveState::DriveBraking,
            };
            DriveStateContext {
                state: next,
                direction,
                state_time: 0.0,
            }
        }

        DriveState::Stopped => {
            let can_leave = state_time >= STOP_DELAY;
            let next = match (throttle, brake, can_leave) {
                (true, false, true) => {
                    return DriveStateContext {
                        state: DriveState::Drive,
                        direction: DriveDirection::Forward,
                        state_time: 0.0,
                    };
                }
                (false, true, true) => {
                    return DriveStateContext {
                        state: DriveState::Drive,
                        direction: DriveDirection::Reverse,
                        state_time: 0.0,
                    };
                }
                (false, false, _) => DriveState::Coast,
                (true, true, _) => DriveState::Coast,
                _ => DriveState::Stopped,
            };
            DriveStateContext {
                state: next,
                direction: ctx.direction,
                state_time,
            }
        }

        // Drive / DriveBraking / Braking delegate to update_drive_state
        _ => {
            let (drive, resist) = match ctx.direction {
                DriveDirection::Forward => (throttle, brake),
                DriveDirection::Reverse => (brake, throttle),
            };
            let next = update_drive_state(ctx.state, drive, resist, near_stop);
            let changed = next != ctx.state;
            DriveStateContext {
                state: next,
                direction: ctx.direction,
                state_time: if changed { 0.0 } else { state_time },
            }
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Car {
    pub chassis: BodyId,
    pub wheels: Vec<WheelData>,
    pub objects: [RenderObject; 5],
    pub debug_arrows: [RenderObject; 4],
    pub geometry: Geometry,
    pub steering_angle: f32,
    pub chassis_position: V3,
    pub chassis_orientation: Q,
    pub drive_state: DriveStateContext,
}

// ----------------------------------------------------------------------------
fn raycast_ground(terrain: &Terrain, origin: V3, max_dist: f32) -> Option<(V3, V3, f32)> {
    let terrain_y = terrain.height_at(origin.x0(), origin.x2());
    let t = origin.x1() - terrain_y;

    // Only discard if the wheel is too far above the ground to make contact.
    // Negative t (wheel center below surface) is kept — it means deep penetration
    // and the solver needs the contact to push the wheel back out.
    if t > max_dist {
        return None;
    }

    let point = V3::new([origin.x0(), terrain_y, origin.x2()]);
    let normal = terrain.normal_at(origin.x0(), origin.x2());

    Some((point, normal, t))
}

// ----------------------------------------------------------------------------
impl Car {
    // ------------------------------------------------------------------------
    pub fn new(context: &mut RenderContext, physics: &mut Physics, geo: Geometry) -> Result<Self> {
        let mut debug_arrows = Vec::new();
        for _ in 0..4 {
            let arrow_verts = arrow(V3::ZERO, V3::X0)?;
            let debug_arrow = RenderObject {
                name: String::from("car:debug_arrow_left"),
                transform: Transform::default(),
                pipe_id: 0,
                mesh_id: context.create_colored_mesh(&arrow_verts, &[], true)?,
                material_id: context.default_material(DefaultMaterials::Green),
                ..Default::default()
            };
            debug_arrows.push(debug_arrow);
        }

        use crate::core::gl_pipeline_colored::{cylinder, transform_mesh};
        let (mut verts, indices) = cylinder(12, geo.wheel_radius, geo.wheel_width);
        transform_mesh(
            &mut verts,
            V3::default(),
            M3x3::from_cols(-V3::X1, V3::X0, V3::X2),
        );
        let wheel_mesh_id = context.create_colored_mesh(&verts, &indices, false)?;
        let chassis_mesh_id = context.default_mesh(DefaultMeshes::Cube);

        // This is temporary and gives the car 952 kg.
        let chassis_material = x2d::WOOD;
        let dimensions = V3::new([geo.width, 0.2, geo.length]);
        let mass = Mass::from_box(chassis_material.density, dimensions)?;

        let chassis_body = RigidBody::new(
            String::from("car:chassis"),
            mass,
            chassis_material,
            V3::new([0.0, 2.0 + geo.wheel_radius + 0.2, 0.0]),
            Q::identity(),
        );

        let wheel_material = x2d::RUBBER;
        let wheel_mass = Mass::from_wheel(wheel_material.density, geo.wheel_radius)?;

        let track_half = 0.5 * geo.wheel_track;
        let base_half = 0.5 * geo.wheel_base;
        let wheels = [
            (true, false, "FL", V3::new([-track_half, 0.0, base_half])),
            (true, false, "FR", V3::new([track_half, 0.0, base_half])),
            (false, true, "RL", V3::new([-track_half, 0.0, -base_half])),
            (false, true, "RR", V3::new([track_half, 0.0, -base_half])),
        ];

        let wheels = wheels
            .iter()
            .map(|(steering, driving, name, local)| {
                let offset = chassis_body.to_world(*local);
                let wheel_body = RigidBody::new(
                    String::from(*name),
                    wheel_mass,
                    wheel_material,
                    offset,
                    Q::identity(),
                );

                (*steering, *driving, *local, wheel_body)
            })
            .collect::<Vec<_>>();

        let chassis_id = physics.add_body(chassis_body);

        let suspension_softness = Softness::new(3.0, 0.2, 1.0 / 100.0);

        let world_basis = M3x3::from_cols(V3::X0, V3::X1, V3::X2);

        let wheels = wheels
            .into_iter()
            .map(|(steering, driving, local, wheel_body)| {
                let wheel_id = physics.add_body(wheel_body);

                let joint = Joint::new_wheel(
                    wheel_id,
                    chassis_id,
                    V3::ZERO,
                    local,
                    world_basis,
                    geo.wheel_radius / 4.0,
                    suspension_softness,
                );

                let joint_id = physics.add_joint(joint);

                WheelData::new(
                    steering,
                    driving,
                    local,
                    wheel_id,
                    joint_id,
                    geo.wheel_radius,
                    geo.wheel_width,
                )
            })
            .collect::<Vec<_>>();

        Ok(Self {
            chassis: chassis_id,
            objects: [
                RenderObject {
                    name: String::from("car:chassis"),
                    transform: Transform {
                        size: V4::new([geo.width, 0.2, geo.length, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: chassis_mesh_id,
                    material_id: context.default_material(DefaultMaterials::White),
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:front_left"),
                    transform: Transform::default(),
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: context.default_material(DefaultMaterials::Black),
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:front_right"),
                    transform: Transform::default(),
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: context.default_material(DefaultMaterials::Black),
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:rear_left"),
                    transform: Transform::default(),
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: context.default_material(DefaultMaterials::Black),
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("car:wheel:rear_right"),
                    transform: Transform::default(),
                    pipe_id: 0,
                    mesh_id: wheel_mesh_id,
                    material_id: context.default_material(DefaultMaterials::Black),
                    ..Default::default()
                },
            ],
            debug_arrows: debug_arrows.try_into().unwrap(),
            wheels,
            geometry: geo,
            steering_angle: 0.0,
            chassis_position: V3::ZERO,
            chassis_orientation: Q::identity(),
            drive_state: DriveStateContext::default(),
        })
    }

    // ------------------------------------------------------------------------
    pub fn position(&self) -> V4 {
        V4::from_v3(self.chassis_position, 1.0)
    }

    // ------------------------------------------------------------------------
    pub fn update_debug_arrows(
        &mut self,
        context: &mut RenderContext,
        physics: &Physics,
    ) -> Result<()> {
        for (wheel_data, render_object) in self.wheels.iter().zip(self.debug_arrows.iter_mut()) {
            let body = physics
                .get_body(wheel_data.body)
                .ok_or(Error::InvalidBodyId)?;

            let joint = physics
                .get_joint(wheel_data.joint)
                .ok_or(Error::InvalidJointId)?;

            let wheel_joint = joint.as_wheel().ok_or(Error::InvalidJointType)?;

            let wheel_pos = body.position();
            //let forward = body.orientation().rotate(V3::X2);
            //let axis = wheel_joint.n[2];
            let axis = wheel_joint.accumulated_lambda[1] * wheel_joint.n[1];

            if let Ok(arrow_verts) = arrow(wheel_pos, wheel_pos - 0.5 * axis) {
                context.update_colored_mesh(render_object.mesh_id, &arrow_verts, &[])?;
            }
        }

        Ok(())
    }

    // ------------------------------------------------------------------------
    pub fn transform(&self, physics: &Physics) -> Result<(V4, V4)> {
        let chassis_body = physics.get_body(self.chassis).ok_or(Error::InvalidBodyId)?;
        let forward = chassis_body.orientation().rotate(V3::X2);
        let position = chassis_body.position();
        Ok((V4::from_v3(forward, 0.0), V4::from_v3(position, 1.0)))
    }

    // ------------------------------------------------------------------------
    pub fn drive_state(&self) -> String {
        format!("{}/{}", self.drive_state.state, self.drive_state.direction)
    }

    // ------------------------------------------------------------------------
    pub fn update(&mut self, ctx: &Context, physics: &mut Physics) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        const DRIVE_TORQUE: f32 = 4000.0;
        const BRAKE_TORQUE: f32 = 2000.0;
        const ENGINE_BRAKE_TORQUE: f32 = 100.0;
        let dt = ctx.dt_secs();

        let throttle = ctx.state.is_pressed(GameKey::Accelerate);
        let brake = ctx.state.is_pressed(GameKey::Brake);

        if ctx.state.is_pressed(GameKey::SteerLeft) {
            self.steering_angle -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(GameKey::SteerRight) {
            self.steering_angle += TURN_SPEED * dt;
        }

        let chassis_body = physics.get_body(self.chassis).ok_or(Error::InvalidBodyId)?;
        let chassis_orientation = chassis_body.orientation();

        let forward = chassis_orientation.rotate(V3::X2);
        let v_long = chassis_body.linear_velocity().dot(forward);

        self.drive_state = update_direction_state(&self.drive_state, throttle, brake, v_long, dt);

        let max_speed = 20.0;
        let (free_speed, free_torque, drive_speed, drive_torque) = match self.drive_state.state {
            DriveState::Coast => (0.0, 0.0, 0.0, ENGINE_BRAKE_TORQUE),
            DriveState::Drive => match self.drive_state.direction {
                DriveDirection::Forward => (0.0, 0.0, -max_speed, DRIVE_TORQUE),
                DriveDirection::Reverse => (0.0, 0.0, max_speed, DRIVE_TORQUE),
            },
            DriveState::DriveBraking => match self.drive_state.direction {
                DriveDirection::Forward => (0.0, BRAKE_TORQUE, -max_speed, DRIVE_TORQUE),
                DriveDirection::Reverse => (0.0, BRAKE_TORQUE, max_speed, DRIVE_TORQUE),
            },
            DriveState::Braking | DriveState::Stopped => (0.0, BRAKE_TORQUE, 0.0, BRAKE_TORQUE),
        };

        for wheel_data in &mut self.wheels {
            let wheel_body = physics
                .get_body(wheel_data.body)
                .ok_or(Error::InvalidBodyId)?;
            let origin = wheel_body.position();

            // Get col0 = lateral (right), col1 = suspension (up), col2 = forward
            let chassis_basis: M3x3 = chassis_orientation.as_mat3x3();

            let joint = physics
                .get_joint_mut(wheel_data.joint)
                .ok_or(Error::InvalidJointId)?;
            let wheel_joint = joint.as_wheel_mut().ok_or(Error::InvalidJointType)?;
            wheel_joint.update_basis(chassis_basis);

            let tire_basis = if wheel_data.is_steering {
                let steering = Q::from_axis_angle(chassis_basis.col1(), self.steering_angle);
                (steering * chassis_orientation).as_mat3x3()
            } else {
                chassis_basis
            };

            if wheel_data.is_driving {
                wheel_joint.update_motor(drive_speed, drive_torque);
            } else {
                wheel_joint.update_motor(free_speed, free_torque);
            }

            if let Some((point, normal, dist)) =
                raycast_ground(ctx.terrain, origin, wheel_data.radius)
            {
                let penetration = wheel_data.radius - dist;
                let normal_force = wheel_joint.normal_force(ctx.dt_secs());

                let tire_contact = TireContext {
                    wheel_radius: wheel_data.radius,
                    contact_point: point,
                    world_basis: tire_basis,
                    normal,
                    penetration,
                    normal_force,
                    friction: 2.8,
                };

                if let Some(contact_id) = wheel_data.contact {
                    if let Some(contact) = physics.get_contact_mut(contact_id) {
                        contact.update(tire_contact);
                    }
                } else {
                    let contact = Contact::new_tire(wheel_data.body, tire_contact);
                    let contact_id = physics.add_contact(contact);
                    wheel_data.contact = Some(contact_id);
                }
            } else {
                if let Some(contact_id) = wheel_data.contact {
                    physics.remove_contact(contact_id);
                    wheel_data.contact = None;
                }
            }
        }

        Ok(())
    }

    // ------------------------------------------------------------------------
    pub fn apply_gravity(&mut self, physics: &mut Physics) -> Result<()> {
        let chassis_body = physics
            .get_body_mut(self.chassis)
            .ok_or(Error::InvalidBodyId)?;

        chassis_body.apply_force(GRAVITY * chassis_body.mass());

        for wheel_data in &self.wheels {
            let wheel_body = physics
                .get_body_mut(wheel_data.body)
                .ok_or(Error::InvalidBodyId)?;

            wheel_body.apply_force(GRAVITY * wheel_body.mass());
        }

        Ok(())
    }

    // ------------------------------------------------------------------------
    pub fn update_render_objects(&mut self, physics: &Physics) -> Result<()> {
        let chassis_body = physics.get_body(self.chassis).ok_or(Error::InvalidBodyId)?;

        self.chassis_position = chassis_body.position();
        self.chassis_orientation = chassis_body.orientation();

        self.objects[0].transform.rotation = self.chassis_orientation.into();
        self.objects[0].transform.position = V4::from_v3(self.chassis_position, 1.0);

        for (wheel_data, render_obj) in self.wheels.iter().zip(self.objects[1..].iter_mut()) {
            let wheel_body = physics
                .get_body(wheel_data.body)
                .ok_or(Error::InvalidBodyId)?;

            render_obj.transform = wheel_body.transform();

            if wheel_data.is_steering {
                let steering = Q::from_axis_angle(V3::X1, self.steering_angle);
                render_obj.transform.rotation = (steering * wheel_body.orientation()).into();
            } else {
                render_obj.transform.rotation = wheel_body.orientation().into();
            }
        }

        Ok(())
    }
}
