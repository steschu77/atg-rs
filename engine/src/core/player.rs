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
#[derive(Debug, Clone, Default)]
pub struct Pose {
    pub body: V3,
    pub head: V3,
    pub feet: [V3; 2],
}

// ----------------------------------------------------------------------------
impl Pose {
    pub fn lerp(&self, target: &Pose, t: f32) -> Pose {
        Pose {
            body: self.body.lerp(&target.body, t),
            head: self.head.lerp(&target.head, t),
            feet: [
                self.feet[0].lerp(&target.feet[0], t),
                self.feet[1].lerp(&target.feet[1], t),
            ],
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Foot {
    Left,
    Right,
}

// ----------------------------------------------------------------------------
impl Foot {
    pub fn index(self) -> usize {
        match self {
            Foot::Left => 0,
            Foot::Right => 1,
        }
    }

    pub fn support(self) -> usize {
        1 - self.index()
    }

    pub fn lateral(self) -> f32 {
        match self {
            Foot::Left => -0.4,
            Foot::Right => 0.4,
        }
    }
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

    pub fn step(&mut self, ctx: &Context, foot: Foot, forward: Option<f32>) {
        self.step_progress = 0.0;
        self.start_pose = self.current_pose.clone();

        let step = foot.index();
        let support = foot.support();

        // place foot 'forward' units ahead of support foot
        let foot_offset = V2::new([foot.lateral(), forward.unwrap_or(0.0)]);

        let support_pos = V2::new([
            self.current_pose.feet[support].x0(),
            self.current_pose.feet[support].x2(),
        ]);

        let foot_pos = support_pos + self.rotation * foot_offset;
        let height = ctx.terrain.height_at(foot_pos.x0(), foot_pos.x1());

        let body_pos = 0.5
            * V2::new([
                foot_pos.x0() + self.current_pose.feet[support].x0(),
                foot_pos.x1() + self.current_pose.feet[support].x2(),
            ]);

        let mut feet = self.current_pose.feet;
        feet[step] = V3::new([foot_pos.x0(), height + 0.1, foot_pos.x1()]);

        self.target_pose = Pose {
            body: V3::new([body_pos.x0(), height + 0.8, body_pos.x1()]),
            head: V3::new([body_pos.x0(), height + 1.8, body_pos.x1()]),
            feet,
        };
    }

    pub fn finish_step(&mut self, ctx: &Context) {
        if ctx.state.is_pressed(input::Key::MoveForward) {
            // Keep walking
            if self.state == AnimationState::SteppingLeft
                || self.state == AnimationState::IntoIdleLeft
            {
                self.state = AnimationState::SteppingRight;
                self.step(ctx, Foot::Right, Some(self.step_length));
                return;
            } else {
                self.state = AnimationState::SteppingLeft;
                self.step(ctx, Foot::Left, Some(self.step_length));
                return;
            }
        }

        // Transition to idle
        match self.state {
            AnimationState::SteppingLeft => {
                self.state = AnimationState::IntoIdleRight;
                self.step(ctx, Foot::Right, None);
            }
            AnimationState::SteppingRight => {
                self.state = AnimationState::IntoIdleLeft;
                self.step(ctx, Foot::Left, None);
            }
            _ => {
                self.idle();
            }
        }
    }

    pub fn idle(&mut self) {
        self.state = AnimationState::Idle;
        self.current_pose = self.target_pose.clone();
        self.step_progress = 0.0;
    }

    pub fn position(&self) -> V4 {
        V4::new([self.position.x0(), 1.8, self.position.x1(), 1.0])
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
        let t = self.step_progress * self.step_speed;

        if ctx.state.is_pressed(input::Key::TurnLeft) {
            self.rotation -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(input::Key::TurnRight) {
            self.rotation += TURN_SPEED * dt;
        }

        match self.state {
            AnimationState::Idle => {
                if ctx.state.is_pressed(input::Key::MoveForward) {
                    self.state = AnimationState::SteppingLeft;
                    self.step(ctx, Foot::Left, Some(self.step_length));
                }
            }
            AnimationState::IntoIdleRight
            | AnimationState::IntoIdleLeft
            | AnimationState::SteppingLeft
            | AnimationState::SteppingRight => {
                // Interpolate position
                let start = &self.start_pose;
                let target = &self.target_pose;
                self.current_pose = start.lerp(target, t);
            }
        }

        let pos = 0.5 * (self.current_pose.feet[0] + self.current_pose.feet[1]);
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
            self.current_pose.feet[0].x0(),
            self.current_pose.feet[0].x1(),
            self.current_pose.feet[0].x2(),
            1.0,
        ]);
        self.objects[3].transform.position = V4::new([
            self.current_pose.feet[1].x0(),
            self.current_pose.feet[1].x1(),
            self.current_pose.feet[1].x2(),
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
