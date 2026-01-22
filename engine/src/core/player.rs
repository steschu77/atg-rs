use crate::core::component::{Component, Context};
use crate::core::gl_renderer::{RenderContext, RenderObject, Transform};
use crate::core::input;
use crate::error::Result;
use crate::v2d::{r2::R2, v2::V2, v3::V3, v4::V4};

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Stepping,
}

// ----------------------------------------------------------------------------
// Animation targets for smooth transitions
#[derive(Debug, Clone)]
pub struct Pose {
    pub body: V3,
    pub head: V3,
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Player {
    pub objects: [RenderObject; 2],
    pub rotation: R2,
    pub position: V2,
    pub state: AnimationState,
    pub current_pose: Pose,
    pub target_pose: Pose,
    pub step_length: f32,
    pub step_height: f32,
    pub step_speed: f32,
    pub step_progress: f32,
}

// ----------------------------------------------------------------------------
impl Player {
    pub fn new(_context: &mut RenderContext) -> Self {
        Self {
            objects: [
                RenderObject {
                    name: String::from("player:body"),
                    transform: Transform {
                        size: V4::new([0.8, 1.0, 0.8, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: 0,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("player:head"),
                    transform: Transform {
                        size: V4::new([1.0, 1.0, 1.0, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: 0,
                    material_id: 0,
                    ..Default::default()
                },
            ],
            rotation: R2::default(),
            position: V2::default(),
            state: AnimationState::Idle,
            current_pose: Pose {
                body: V3::default(),
                head: V3::default(),
            },
            target_pose: Pose {
                body: V3::default(),
                head: V3::default(),
            },
            step_length: 0.8,
            step_height: 0.1,
            step_speed: 4.0,
            step_progress: 0.0,
        }
    }

    pub fn start_step(&mut self, ctx: &Context) {
        self.state = AnimationState::Stepping;
        self.step_progress = 0.0;

        let direction = self.rotation.x_axis();

        let pos = self.position + direction * self.step_length;
        let height = ctx.terrain.height_at(pos.x0(), pos.x1());

        self.target_pose.body = V3::new([pos.x0(), height + 0.5, pos.x1()]);
        self.target_pose.head = V3::new([pos.x0(), height + 1.6, pos.x1()]);
    }

    pub fn finish_step(&mut self) {
        self.state = AnimationState::Idle;
        self.current_pose = self.target_pose.clone();
    }
}

// ----------------------------------------------------------------------------
impl Component for Player {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();

        if ctx.state.is_pressed(input::Key::TurnLeft) {
            self.rotation -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(input::Key::TurnRight) {
            self.rotation += TURN_SPEED * dt;
        }

        match self.state {
            AnimationState::Idle => {
                if ctx.state.is_pressed(input::Key::MoveForward) {
                    self.start_step(ctx);
                }
            }
            AnimationState::Stepping => {
                self.step_progress += dt;

                let t = self.step_progress * self.step_speed;
                if t >= 1.0 {
                    self.finish_step();
                } else {
                    // Interpolate position
                    self.current_pose.body = self.current_pose.body.lerp(&self.target_pose.body, t);
                    self.current_pose.head = self.current_pose.head.lerp(&self.target_pose.head, t);

                    // Add step height (parabolic)
                    let height_offset = 4.0 * self.step_height * t * (1.0 - t);
                    self.current_pose.body = V3::new([
                        self.current_pose.body.x0(),
                        self.current_pose.body.x1() + height_offset,
                        self.current_pose.body.x2(),
                    ]);
                }
            }
        }

        self.position = V2::new([self.current_pose.body.x0(), self.current_pose.body.x2()]);
        self.objects[0].transform.position = V4::new([
            self.current_pose.body.x0(),
            self.current_pose.body.x1(),
            self.current_pose.body.x2(),
            1.0,
        ]);
        self.objects[1].transform.position = V4::new([
            self.current_pose.head.x0(),
            self.current_pose.head.x1(),
            self.current_pose.head.x2(),
            1.0,
        ]);

        let rotation = self.rotation.get();
        let rotation = V4::new([0.0, rotation, 0.0, 0.0]);
        self.objects[0].transform.rotation = rotation;
        self.objects[1].transform.rotation = rotation;
        Ok(())
    }
}
