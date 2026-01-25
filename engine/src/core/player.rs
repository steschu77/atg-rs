use crate::core::component::{Component, Context};
use crate::core::gl_renderer::{RenderContext, RenderObject, Transform};
use crate::core::input;
use crate::error::Result;
use crate::v2d::{r2::R2, v2::V2, v3::V3, v4::V4};

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    SteppingLeft,
    SteppingRight,
    IntoIdleLeft,
    IntoIdleRight,
}

// ----------------------------------------------------------------------------
// Animation targets for smooth transitions
#[derive(Debug, Clone, Default)]
pub struct Pose {
    pub body: V3,
    pub head: V3,
    pub foot_left: V3,
    pub foot_right: V3,
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Player {
    pub objects: [RenderObject; 4],
    pub rotation: R2,
    pub position: V2,
    pub state: AnimationState,
    pub current_pose: Pose,
    pub start_pose: Pose,
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
                        size: V4::new([0.8, 0.8, 0.5, 1.0]),
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
                        size: V4::new([0.6, 0.6, 0.6, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: 0,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("player:foot_left"),
                    transform: Transform {
                        size: V4::new([0.3, 0.2, 0.4, 1.0]),
                        ..Default::default()
                    },
                    pipe_id: 0,
                    mesh_id: 0,
                    material_id: 0,
                    ..Default::default()
                },
                RenderObject {
                    name: String::from("player:foot_right"),
                    transform: Transform {
                        size: V4::new([0.3, 0.2, 0.4, 1.0]),
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
            current_pose: Pose::default(),
            start_pose: Pose::default(),
            target_pose: Pose::default(),
            step_length: 0.8,
            step_height: 0.1,
            step_speed: 4.0,
            step_progress: 0.0,
        }
    }

    pub fn start_step(&mut self, ctx: &Context, state: AnimationState) {
        self.state = state;
        self.step_progress = 0.0;
        self.start_pose = self.current_pose.clone();

        let step_offset = if state == AnimationState::SteppingLeft {
            0.2
        } else {
            -0.2
        };

        let x_axis = self.rotation.x_axis();
        let y_axis = self.rotation.y_axis();

        let body_pos = self.position + y_axis * self.step_length * 0.5;
        let foot_pos = self.position + y_axis * self.step_length + x_axis * step_offset;
        let height = ctx.terrain.height_at(foot_pos.x0(), foot_pos.x1());

        self.target_pose.body = V3::new([body_pos.x0(), height + 0.8, body_pos.x1()]);
        self.target_pose.head = V3::new([body_pos.x0(), height + 1.8, body_pos.x1()]);

        if state == AnimationState::SteppingLeft {
            self.target_pose.foot_left = V3::new([foot_pos.x0(), height + 0.1, foot_pos.x1()]);
            self.target_pose.foot_right = self.current_pose.foot_right;
        } else {
            self.target_pose.foot_right = V3::new([foot_pos.x0(), height + 0.1, foot_pos.x1()]);
            self.target_pose.foot_left = self.current_pose.foot_left;
        };
    }

    pub fn to_idle(&mut self, ctx: &Context, state: AnimationState) {
        self.state = state;
        self.step_progress = 0.0;
        self.start_pose = self.current_pose.clone();

        let step_offset = if state == AnimationState::IntoIdleLeft {
            0.2
        } else {
            -0.2
        };

        let x_axis = self.rotation.x_axis();
        let y_axis = self.rotation.y_axis();

        let body_pos = self.position + y_axis * self.step_length * 0.5;
        let foot_pos = self.position + y_axis * self.step_length * 0.5 + x_axis * step_offset;
        let height = ctx.terrain.height_at(foot_pos.x0(), foot_pos.x1());

        self.target_pose.body = V3::new([body_pos.x0(), height + 0.6, body_pos.x1()]);
        self.target_pose.head = V3::new([body_pos.x0(), height + 1.6, body_pos.x1()]);

        if state == AnimationState::IntoIdleLeft {
            self.target_pose.foot_left = V3::new([foot_pos.x0(), height + 0.1, foot_pos.x1()]);
            self.target_pose.foot_right = self.current_pose.foot_right;
        } else {
            self.target_pose.foot_right = V3::new([foot_pos.x0(), height + 0.1, foot_pos.x1()]);
            self.target_pose.foot_left = self.current_pose.foot_left;
        };
    }

    pub fn finish_step(&mut self, ctx: &Context) {
        if ctx.state.is_pressed(input::Key::MoveForward) {
            // Keep walking
            if self.state == AnimationState::SteppingLeft
                || self.state == AnimationState::IntoIdleLeft
            {
                self.start_step(ctx, AnimationState::SteppingRight);
                return;
            }
            if self.state == AnimationState::SteppingRight
                || self.state == AnimationState::IntoIdleRight
            {
                self.start_step(ctx, AnimationState::SteppingLeft);
                return;
            }
        }

        // Transition to idle
        match self.state {
            AnimationState::SteppingLeft => {
                self.to_idle(ctx, AnimationState::IntoIdleRight);
                return;
            }
            AnimationState::SteppingRight => {
                self.to_idle(ctx, AnimationState::IntoIdleLeft);
                return;
            }
            AnimationState::IntoIdleRight | AnimationState::IntoIdleLeft => {
                self.idle();
                return;
            }
            _ => {}
        }
        self.step_progress = 0.0;
    }

    pub fn idle(&mut self) {
        self.state = AnimationState::Idle;
        self.current_pose = self.target_pose.clone();
        self.step_progress = 0.0;
    }
}

// ----------------------------------------------------------------------------
impl Component for Player {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();
        self.step_progress += dt;

        let t = self.step_progress * self.step_speed;
        if t >= 1.0 {
            self.finish_step(ctx);
        }

        if ctx.state.is_pressed(input::Key::TurnLeft) {
            self.rotation -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(input::Key::TurnRight) {
            self.rotation += TURN_SPEED * dt;
        }

        match self.state {
            AnimationState::Idle => {
                if ctx.state.is_pressed(input::Key::MoveForward) {
                    self.start_step(ctx, AnimationState::SteppingLeft);
                }
            }
            AnimationState::IntoIdleRight
            | AnimationState::IntoIdleLeft
            | AnimationState::SteppingLeft
            | AnimationState::SteppingRight => {
                // Interpolate position
                let start = &self.start_pose;
                let target = &self.target_pose;
                let current = &mut self.current_pose;
                current.body = start.body.lerp(&target.body, t);
                current.head = start.head.lerp(&target.head, t);
                current.foot_left = start.foot_left.lerp(&target.foot_left, t);
                current.foot_right = start.foot_right.lerp(&target.foot_right, t);
            }
        }

        let pos = self
            .current_pose
            .foot_left
            .lerp(&self.current_pose.foot_right, 0.5);
        self.position = V2::new([pos.x0(), pos.x2()]);

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
        self.objects[2].transform.position = V4::new([
            self.current_pose.foot_left.x0(),
            self.current_pose.foot_left.x1(),
            self.current_pose.foot_left.x2(),
            1.0,
        ]);
        self.objects[3].transform.position = V4::new([
            self.current_pose.foot_right.x0(),
            self.current_pose.foot_right.x1(),
            self.current_pose.foot_right.x2(),
            1.0,
        ]);

        let rotation = self.rotation.get();
        let rotation = V4::new([0.0, rotation, 0.0, 0.0]);
        self.objects[0].transform.rotation = rotation;
        self.objects[1].transform.rotation = rotation;
        self.objects[2].transform.rotation = rotation;
        self.objects[3].transform.rotation = rotation;
        Ok(())
    }
}
