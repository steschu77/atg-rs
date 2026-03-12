use crate::core::component::Context;
use crate::core::game_input::GameKey;
use crate::core::gl_pipeline_colored::{arrow, cylinder, transform_mesh};
use crate::core::gl_renderer::{DefaultMaterials, RenderContext, RenderObject, Transform};
use crate::error::{Error, Result};
use crate::v2d::{affine3x3, m3x3::M3x3, q::Q, v3::V3, v4::V4};
use crate::x2d::{
    self, BodyId, ContactId, constraint::contact::Contact, constraint::tire_contact::TireContext,
    mass::Mass, physics::Physics, rigid_body::RigidBody,
};

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct Wheel {
    pub radius: f32,
    pub width: f32,
    pub position: V3,
    pub basis: M3x3,
    pub steering_angle: f32,
    pub spin_angle: f32,
    pub drive_torque: f32,
    pub brake_torque: f32,
    pub body_id: BodyId,
    pub contact_id: Option<ContactId>,
    pub object: RenderObject,
    pub debug_arrows: [RenderObject; 2],
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
impl Wheel {
    pub fn new(context: &mut RenderContext, physics: &mut Physics, position: V3) -> Result<Self> {
        let wheel_radius = 0.4;
        let wheel_width = 0.3;

        let left_arrow_mesh_id = context.create_colored_mesh(&[], &[], true)?;
        let right_arrow_mesh_id = context.create_colored_mesh(&[], &[], true)?;

        let (mut verts, indices) = cylinder(12, wheel_radius, wheel_width);
        transform_mesh(
            &mut verts,
            V3::default(),
            M3x3::from_cols(-V3::X1, V3::X0, V3::X2),
        );
        let wheel_mesh_id = context.create_colored_mesh(&verts, &indices, false)?;
        let wheel_object = RenderObject {
            name: String::from("wheel"),
            transform: Transform::default(),
            pipe_id: 0,
            mesh_id: wheel_mesh_id,
            material_id: context.default_material(DefaultMaterials::Green),
            ..Default::default()
        };
        let debug_arrows = [
            RenderObject {
                name: String::from("wheel:debug_arrow_left"),
                transform: Transform::default(),
                pipe_id: 0,
                mesh_id: left_arrow_mesh_id,
                material_id: context.default_material(DefaultMaterials::Green),
                ..Default::default()
            },
            RenderObject {
                name: String::from("wheel:debug_arrow_right"),
                transform: Transform::default(),
                pipe_id: 0,
                mesh_id: right_arrow_mesh_id,
                material_id: context.default_material(DefaultMaterials::Green),
                ..Default::default()
            },
        ];

        let wheel_material = x2d::RUBBER;
        let wheel_mass = Mass::from_wheel(wheel_material.density, wheel_radius)?;
        let mut wheel_body = RigidBody::new(wheel_mass, wheel_material, position, Q::identity());
        wheel_body.apply_angular_impulse(V3::X0 * 1000.0, "initial_impulse");
        let body_id = physics.add_body(wheel_body);

        Ok(Self {
            radius: wheel_radius,
            width: wheel_width,
            position,
            basis: M3x3::identity(),
            steering_angle: 0.0,
            spin_angle: 0.0,
            drive_torque: 0.0,
            brake_torque: 0.0,
            body_id,
            contact_id: None,
            object: wheel_object,
            debug_arrows,
        })
    }

    pub fn id(&self) -> BodyId {
        self.body_id
    }

    pub fn position(&self) -> V4 {
        self.object.transform.position
    }

    pub fn orientation(&self) -> M3x3 {
        M3x3::identity()
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        let wheel_pos = self.position().into();
        for i in 0..2 {
            let forward = self.basis.col2();
            let arrow_verts = arrow(wheel_pos, wheel_pos + 1.5 * forward)?;
            context.update_colored_mesh(self.debug_arrows[i].mesh_id, &arrow_verts, &[])?;
        }

        Ok(())
    }

    pub fn transform(&self) -> (V4, V4) {
        let forward = self.orientation().col2();
        let position = self.position();
        (V4::from_v3(forward, 0.0), position)
    }
}

// ----------------------------------------------------------------------------
impl Wheel {
    pub fn update(&mut self, ctx: &Context, physics: &mut Physics, dt: f32) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();

        self.drive_torque = if ctx.state.is_pressed(GameKey::Accelerate) {
            1200.0
        } else {
            0.0
        };

        self.brake_torque = if ctx.state.is_pressed(GameKey::Brake) {
            1500.0
        } else {
            0.0
        };

        if ctx.state.is_pressed(GameKey::SteerLeft) {
            self.steering_angle -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(GameKey::SteerRight) {
            self.steering_angle += TURN_SPEED * dt;
        }

        let wheel_body = physics.get_body(self.body_id).unwrap();

        let origin = wheel_body.position();
        let orientation = wheel_body.orientation();
        let dir = -V3::X1;

        if let Some((point, normal, dist)) = raycast_ground(origin, dir, self.radius) {
            let forward = orientation.rotate(V3::X2);
            let forward = (forward - normal * forward.dot(normal)).norm();
            let lateral = normal.cross(forward).norm();
            let basis = M3x3::from_cols(lateral, normal, forward);

            let penetration = self.radius - dist;

            let tire_contact = TireContext {
                wheel_radius: self.radius,
                contact_point: point,
                basis,
                penetration,
                normal_force: 100.0,
                friction: 0.8,
                drive_torque: self.drive_torque,
                brake_torque: self.brake_torque,
            };

            if let Some(contact_id) = self.contact_id {
                if let Some(contact) = physics.get_contact_mut(contact_id) {
                    contact.update(tire_contact);
                }
            } else {
                let contact = Contact::new_tire(self.body_id, tire_contact);
                let contact_id = physics.add_contact(contact);
                self.contact_id = Some(contact_id);
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if let Some(contact_id) = self.contact_id {
                physics.remove_contact(contact_id);
                self.contact_id = None;
            }
        }

        let body = physics.get_body(self.body_id).unwrap();
        let angular_vel = body.angular_velocity();
        self.spin_angle += angular_vel.x0() * dt;

        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Wheel {
    pub fn update_render_objects(&mut self, physics: &Physics) -> Result<()> {
        let body = physics.get_body(self.body_id).ok_or(Error::InvalidBodyId)?;

        let transform = body.transform();
        self.object.transform.position = transform.position;
        self.object.transform.rotation = transform.rotation;
        Ok(())
    }
}
