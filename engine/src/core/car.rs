use crate::core::component::{Component, Context};
use crate::core::game_input::GameKey;
use crate::core::gl_renderer::{RenderContext, RenderObject, Rotation, Transform};
use crate::error::Result;
use crate::v2d::{m3x3::M3x3, r2::R2, v3::V3, v4::V4};

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
#[derive(Debug, Clone, Default)]
pub struct ChassisData {
    pub position: V3,
    pub velocity: V3,
    pub mass: f32,
    pub rotation: R2,
    pub angular_velocity: V3,
    pub inertia: V3,
    pub steering_angle: f32,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct WheelData {
    pub position: V3,
    pub radius: f32,
    pub width: f32,
    pub rotation: R2,
    pub suspension_length: f32,
    pub spring_constant: f32,
    pub damping_coefficient: f32,
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
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Car {
    pub objects: [RenderObject; 5],
    pub debug_arrows: [RenderObject; 2],
    pub chassis: ChassisData,
    pub wheels: [WheelData; 4],
    pub geometry: Geometry,
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

        Ok(Self {
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
                    name: String::from("player:debug_arrow_left"),
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
                    name: String::from("player:debug_arrow_right"),
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
                position: V3::new([0.0, geo.wheel_radius + 0.2, 0.0]),
                mass: 1200.0,
                inertia: V3::new([1500.0, 1500.0, 1500.0]),
                ..Default::default()
            },
            wheels: [
                WheelData {
                    position: V3::new([-0.5 * geo.wheel_track, 0.0, 0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
                WheelData {
                    position: V3::new([0.5 * geo.wheel_track, 0.0, 0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
                WheelData {
                    position: V3::new([-0.5 * geo.wheel_track, 0.0, -0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
                WheelData {
                    position: V3::new([0.5 * geo.wheel_track, 0.0, -0.5 * geo.wheel_base]),
                    radius: geo.wheel_radius,
                    width: geo.wheel_width,
                    ..Default::default()
                },
            ],
            geometry: geo,
        })
    }

    pub fn position(&self) -> V4 {
        let pos = self.chassis.position;
        V4::new([pos.x0(), pos.x1(), pos.x2(), 1.0])
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        use crate::core::gl_pipeline_colored::arrow;

        for i in 0..2 {
            let wheel_pos = self.chassis.position + self.wheels[i].position;
            let forward = self.wheels[i].rotation.y_axis();
            let forward = V3::new([forward.x0(), 0.0, forward.x1()]);
            let arrow_verts = arrow(wheel_pos, forward, 1.5);
            context.update_colored_mesh(self.debug_arrows[i].mesh_id, &arrow_verts, &[])?;
        }

        Ok(())
    }

    pub fn transform(&self) -> (V4, V4) {
        let y_axis = self.chassis.rotation.y_axis();
        let forward = V4::new([y_axis.x0(), 0.0, y_axis.x1(), 0.0]);
        let position = V4::new([
            self.chassis.position.x0(),
            self.chassis.position.x1(),
            self.chassis.position.x2(),
            1.0,
        ]);
        (forward, position)
    }
}

// ----------------------------------------------------------------------------
impl Component for Car {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();

        let accelerate = ctx.state.is_pressed(GameKey::Accelerate);
        let brake = ctx.state.is_pressed(GameKey::Brake);
        if accelerate {
            self.chassis.velocity += V3::new([
                0.0,
                0.0,
                50.0 * dt, // forward in local space
            ]);
        }
        if brake {
            self.chassis.velocity -= V3::new([
                0.0,
                0.0,
                50.0 * dt, // forward in local space
            ]);
        }
        self.chassis.position += self.chassis.velocity * dt;

        if ctx.state.is_pressed(GameKey::SteerLeft) {
            self.chassis.steering_angle -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(GameKey::SteerRight) {
            self.chassis.steering_angle += TURN_SPEED * dt;
        }

        self.objects[0].transform.position = V4::new([
            self.chassis.position.x0(),
            self.chassis.position.x1(),
            self.chassis.position.x2(),
            1.0,
        ]);

        let rotation = self.chassis.rotation.get();
        let rotation = Rotation::Euler(V3::new([0.0, rotation, 0.0]));
        self.objects[0].transform.rotation = rotation;

        for (i, wheel) in &mut self.wheels.iter().enumerate() {
            let steering_angle = if WheelPos::from(i).is_front() {
                self.chassis.steering_angle
            } else {
                0.0
            };
            let wheel_pos = self.chassis.position + wheel.position;
            let wheel_rot = Rotation::Euler(V3::new([wheel.rotation.get(), steering_angle, 0.0]));
            self.objects[1 + i].transform.position =
                V4::new([wheel_pos.x0(), wheel_pos.x1(), wheel_pos.x2(), 1.0]);
            self.objects[1 + i].transform.rotation = wheel_rot;
        }

        Ok(())
    }
}
