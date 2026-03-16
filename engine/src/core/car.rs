use crate::core::component::Context;
use crate::core::game_input::GameKey;
use crate::core::gl_pipeline_colored::arrow;
use crate::core::gl_renderer::{
    DefaultMaterials, DefaultMeshes, RenderContext, RenderObject, Transform,
};
use crate::error::{Error, Result};
use crate::v2d::{m3x3::M3x3, q::Q, v3::V3, v4::V4};
use crate::x2d::{
    self, BodyId, ContactId, JointId, constraint::contact::Contact, constraint::joint::Joint,
    constraint::softness::Softness, constraint::tire_contact::TireContext, mass::Mass,
    physics::Physics, rigid_body::RigidBody,
};

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
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WheelPos {
    FL,
    FR,
    RL,
    RR,
}

// ----------------------------------------------------------------------------
impl TryFrom<usize> for WheelPos {
    type Error = Error;

    fn try_from(index: usize) -> Result<Self> {
        match index {
            0 => Ok(WheelPos::FL),
            1 => Ok(WheelPos::FR),
            2 => Ok(WheelPos::RL),
            3 => Ok(WheelPos::RR),
            _ => Err(Error::InvalidIndex { index }),
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
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct WheelData {
    pub wheel: WheelPos,
    pub local_position: V3,
    pub radius: f32,
    pub width: f32,
    pub drive_torque: f32,
    pub brake_torque: f32,
    pub normal_force: f32,
    pub body: BodyId,
    pub wheel_joint: JointId,
    pub contact_id: Option<ContactId>,
}

// ----------------------------------------------------------------------------
impl WheelData {
    fn new(
        wheel: WheelPos,
        local_position: V3,
        body: BodyId,
        wheel_joint: JointId,
        radius: f32,
        width: f32,
    ) -> Self {
        Self {
            wheel,
            local_position,
            radius,
            width,
            drive_torque: 0.0,
            brake_torque: 0.0,
            normal_force: 0.0,
            body,
            wheel_joint,
            contact_id: None,
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
    pub drive_torque: f32,
    pub brake_torque: f32,
    pub chassis_position: V3,
    pub chassis_orientation: Q,
}

// ----------------------------------------------------------------------------
fn wheel_basis_static(orientation: Q) -> (V3, V3) {
    let forward = orientation.rotate(V3::X2);
    let right = orientation.rotate(V3::X0);
    (forward, right)
}

// ----------------------------------------------------------------------------
fn wheel_basis_steering(orientation: Q, steer_angle: f32) -> (V3, V3) {
    let car_forward = orientation.rotate(V3::X2);
    let car_up = orientation.rotate(V3::X1);

    let steer_q = Q::from_axis_angle(car_up, steer_angle);
    let forward = steer_q.rotate(car_forward).norm();
    let right = car_up.cross(forward).norm();

    (forward, right)
}

// ----------------------------------------------------------------------------
// Simple raycast for ground plane at y=0. Sophisticated terrain raycasting comes later.
fn raycast_ground(origin: V3, dir: V3, max_dist: f32) -> Option<(V3, V3, f32)> {
    if dir.x1() >= 0.0 {
        return None;
    }

    let t = -origin.x1() / dir.x1();

    if t < 0.0 || t > max_dist {
        return None;
    }

    let point = origin + dir * t;

    Some((point, V3::X1, t))
}

// ----------------------------------------------------------------------------
impl Car {
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
            (WheelPos::FL, V3::new([-track_half, 0.0, base_half])),
            (WheelPos::FR, V3::new([track_half, 0.0, base_half])),
            (WheelPos::RL, V3::new([-track_half, 0.0, -base_half])),
            (WheelPos::RR, V3::new([track_half, 0.0, -base_half])),
        ];

        let wheels = wheels
            .iter()
            .map(|(wheel, local)| {
                let offset = chassis_body.to_world(*local);
                let name = format!("car:{wheel:?}");
                let wheel_body =
                    RigidBody::new(name, wheel_mass, wheel_material, offset, Q::identity());

                (*wheel, *local, wheel_body)
            })
            .collect::<Vec<_>>();

        let chassis_id = physics.add_body(chassis_body);

        let suspension_softness = Softness::new(3.0, 0.2, 1.0 / 100.0);

        let world_basis = M3x3::from_cols(V3::X0, V3::X1, V3::X2);

        let wheels = wheels
            .into_iter()
            .map(|(wheel, local, wheel_body)| {
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
                    wheel,
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
            drive_torque: 0.0,
            brake_torque: 0.0,
            steering_angle: 0.0,
            chassis_position: V3::ZERO,
            chassis_orientation: Q::identity(),
        })
    }

    pub fn position(&self) -> V4 {
        V4::from_v3(self.chassis_position, 1.0)
    }

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
                .get_joint(wheel_data.wheel_joint)
                .ok_or(Error::InvalidJointId)?;

            let (_, _, wheel_joint) = joint.as_wheel().ok_or(Error::InvalidJointType)?;

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

    pub fn transform(&self, physics: &Physics) -> Result<(V4, V4)> {
        let chassis_body = physics.get_body(self.chassis).ok_or(Error::InvalidBodyId)?;
        let forward = chassis_body.orientation().rotate(V3::X2);
        let position = chassis_body.position();
        Ok((V4::from_v3(forward, 0.0), V4::from_v3(position, 1.0)))
    }
}

// ----------------------------------------------------------------------------
impl Car {
    pub fn update(&mut self, ctx: &Context, physics: &mut Physics) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();

        self.drive_torque = if ctx.state.is_pressed(GameKey::Accelerate) {
            2000.0 // Nm
        } else {
            0.0
        };

        self.brake_torque = if ctx.state.is_pressed(GameKey::Brake) {
            1000.0 // Nm
        } else {
            0.0
        };

        if ctx.state.is_pressed(GameKey::SteerLeft) {
            self.steering_angle -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(GameKey::SteerRight) {
            self.steering_angle += TURN_SPEED * dt;
        }

        let chassis_body = physics.get_body(self.chassis).ok_or(Error::InvalidBodyId)?;
        let chassis_orientation = chassis_body.orientation();

        for wheel_data in &mut self.wheels {
            let wheel_body = physics
                .get_body(wheel_data.body)
                .ok_or(Error::InvalidBodyId)?;
            let origin = wheel_body.position();

            let lateral = chassis_orientation.rotate(V3::X0).norm();
            let suspension = chassis_orientation.rotate(V3::X1).norm();
            let forward = chassis_orientation.rotate(V3::X2).norm();

            let basis = M3x3::from_cols(lateral, suspension, forward);

            let wheel_joint = physics
                .get_joint_mut(wheel_data.wheel_joint)
                .ok_or(Error::InvalidJointId)?;
            wheel_joint.update_basis(basis);

            let basis = if wheel_data.wheel.is_front() {
                let steering = Q::from_axis_angle(suspension, self.steering_angle);
                let lateral = steering.rotate(lateral);
                let forward = steering.rotate(forward);
                M3x3::from_cols(lateral, suspension, forward)
            } else {
                M3x3::from_cols(lateral, suspension, forward)
            };

            if wheel_data.wheel.is_rear() {
                let wheel_joint = physics
                    .get_joint_mut(wheel_data.wheel_joint)
                    .ok_or(Error::InvalidJointId)?;
                wheel_joint.update_motor(-4.0, self.drive_torque);
            }

            let dir = -V3::X1;
            if let Some((point, normal, dist)) = raycast_ground(origin, dir, wheel_data.radius) {
                let penetration = wheel_data.radius - dist;

                let tire_contact = TireContext {
                    wheel_radius: wheel_data.radius,
                    contact_point: point,
                    world_basis: basis,
                    normal,
                    penetration,
                    normal_force: 6000.0,
                    friction: 0.8,
                };

                if let Some(contact_id) = wheel_data.contact_id {
                    if let Some(contact) = physics.get_contact_mut(contact_id) {
                        contact.update(tire_contact);
                    }
                } else {
                    let contact = Contact::new_tire(wheel_data.body, tire_contact);
                    let contact_id = physics.add_contact(contact);
                    wheel_data.contact_id = Some(contact_id);
                }
            } else {
                if let Some(contact_id) = wheel_data.contact_id {
                    physics.remove_contact(contact_id);
                    wheel_data.contact_id = None;
                }
            }
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Car {
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

            if wheel_data.wheel.is_front() {
                let steering = Q::from_axis_angle(V3::X1, self.steering_angle);
                render_obj.transform.rotation = (steering * wheel_body.orientation()).into();
            } else {
                render_obj.transform.rotation = wheel_body.orientation().into();
            }
        }

        Ok(())
    }
}
