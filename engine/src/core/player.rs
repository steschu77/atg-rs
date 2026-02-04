use crate::core::component::{Component, Context};
use crate::core::game_input::GameKey;
use crate::core::gl_renderer::{RenderContext, RenderObject, Rotation, Transform};
use crate::error::Result;
use crate::v2d::q::Q;
use crate::v2d::{affine4x4, r2::R2, v2::V2, v3::V3, v4::V4};

// ----------------------------------------------------------------------------
// Terminology based on
// https://www.bostonoandp.com/Customer-Content/www/CMS/files/GaitTerminology.pdf

// ----------------------------------------------------------------------------
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    #[default]
    Idle,
    Stepping,
    Closing,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct Skeleton {
    pub body_height: f32,
    pub head_height: f32,
    pub feet_height: f32,
    pub feet_distance: f32,
    pub step_length: f32,
    pub step_height: f32,
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
pub struct Pose {
    pub body: V3,
    pub head: V3,
    pub feet: [V3; 2],
    pub toes: [Q; 2],
    pub toe_dirs: [V3; 2],
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
            toes: [
                self.toes[0].slerp(&target.toes[0], t),
                self.toes[1].slerp(&target.toes[1], t),
            ],
            toe_dirs: [
                self.toe_dirs[0].lerp(&target.toe_dirs[0], t),
                self.toe_dirs[1].lerp(&target.toe_dirs[1], t),
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
    pub fn other(self) -> Foot {
        match self {
            Foot::Left => Foot::Right,
            Foot::Right => Foot::Left,
        }
    }

    pub fn index_self(self) -> usize {
        match self {
            Foot::Left => 0,
            Foot::Right => 1,
        }
    }

    pub fn index_other(self) -> usize {
        1 - self.index_self()
    }

    pub fn side(self) -> f32 {
        match self {
            Foot::Left => -1.0,
            Foot::Right => 1.0,
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StepIntent {
    Advance, // continue walking
    Close,   // bring feet together and stop
}

// ----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StepResult {
    Idle,
    Advance(Foot),
    Close(Foot),
}

// ----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct StepAnimation {
    pub foot: Foot,
    pub intent: StepIntent,
    pub foot_start: V3,
    pub foot_target: V3,
    pub foot_control: V3, // Bézier midpoint
    pub body_bob_height: f32,
    pub toe_roll_max: f32, // radians
}

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Player {
    pub objects: [RenderObject; 4],
    pub debug_arrows: [RenderObject; 2],
    pub rotation: R2,
    pub position: V2,
    pub state: AnimationState,
    pub active_step: Option<StepAnimation>,
    pub current_pose: Pose,
    pub start_pose: Pose,
    pub target_pose: Pose,
    pub step_speed: f32,
    pub phase_progress: f32,
    pub skeleton: Skeleton,
}

// ----------------------------------------------------------------------------
fn bezier_quad(p0: V3, p1: V3, p2: V3, t: f32) -> V3 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

// ----------------------------------------------------------------------------
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 == edge1 {
        return 0.0; // Avoid division by zero
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// ----------------------------------------------------------------------------
fn body_bob(t: f32) -> f32 {
    // Smooth compression then rise, peaks at mid-step
    let x = 2.0 * t - 1.0;
    1.0 - x * x
}

// ----------------------------------------------------------------------------
fn toe_roll(t: f32) -> f32 {
    if t < 0.5 {
        // heel down quickly
        smoothstep(0.0, 0.5, t)
    } else {
        // push off slower
        1.0 - smoothstep(0.5, 1.0, t)
    }
}

// ----------------------------------------------------------------------------
impl Player {
    pub fn new(context: &mut RenderContext) -> Self {
        use crate::core::gl_pipeline_colored::arrow;
        let pos = V3::new([1.0, 0.0, 0.0]);
        let forward_3d = V3::new([0.0, 0.0, 1.0]);
        let arrow_verts = arrow(pos, forward_3d, 1.5);

        let left_arrow_mesh_id = context
            .create_colored_mesh(&arrow_verts, &[], true)
            .unwrap();
        let right_arrow_mesh_id = context
            .create_colored_mesh(&arrow_verts, &[], true)
            .unwrap();
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
            rotation: R2::new(std::f32::consts::FRAC_PI_4),
            position: V2::default(),
            state: AnimationState::Idle,
            active_step: None,
            current_pose: Pose::default(),
            start_pose: Pose::default(),
            target_pose: Pose::default(),
            step_speed: 4.0,
            phase_progress: 0.0,
            skeleton: Skeleton {
                body_height: 0.8,
                head_height: 1.8,
                feet_height: 0.1,
                feet_distance: 0.4,
                step_length: 0.8,
                step_height: 0.3,
            },
        }
    }

    pub fn idle(&mut self) {
        self.phase_progress = 0.0;
        self.start_pose = self.current_pose.clone();
        self.current_pose = self.target_pose.clone();
    }

    pub fn step(&mut self, ctx: &Context, foot: Foot, intent: StepIntent) {
        let Skeleton {
            body_height,
            head_height,
            feet_height,
            feet_distance,
            step_length,
            step_height,
        } = self.skeleton;

        self.phase_progress = 0.0;
        self.start_pose = self.current_pose.clone();

        let swing_foot = foot.index_self();
        let stance_foot = foot.index_other();

        // place foot 'forward' units ahead of support foot
        let (forward, lift, bob, toe_roll_max) = match intent {
            StepIntent::Advance => (step_length, step_height, 0.04, 0.3),
            StepIntent::Close => (0.0, 0.4 * step_height, 0.02, 0.1),
        };
        let foot_offset = V2::new([foot.side() * feet_distance, forward]);

        let stance_pos = V2::new([
            self.current_pose.feet[stance_foot].x0(),
            self.current_pose.feet[stance_foot].x2(),
        ]);

        let foot_pos = stance_pos + self.rotation * foot_offset;
        let height = ctx.terrain.height_at(foot_pos.x0(), foot_pos.x1());
        let normal = ctx.terrain.normal_at(foot_pos.x0(), foot_pos.x1());

        let body_pos = 0.5
            * V2::new([
                foot_pos.x0() + self.current_pose.feet[stance_foot].x0(),
                foot_pos.x1() + self.current_pose.feet[stance_foot].x2(),
            ]);

        let start = self.current_pose.feet[swing_foot];
        let target = V3::new([foot_pos.x0(), height + feet_height, foot_pos.x1()]);
        let control = 0.5 * (start + target) + V3::new([0.0, lift, 0.0]);

        let walk_dir = self.rotation.y_axis();
        let walk_dir = V3::new([walk_dir.x0(), 0.0, walk_dir.x1()]).norm();

        let right = V3::cross(&normal, &walk_dir).norm();
        let toe_dir = V3::cross(&right, &normal).norm();
        let toe = Q::from_axes(&right, &normal, &toe_dir);

        let mut toe_dirs = self.current_pose.toe_dirs;
        toe_dirs[swing_foot] = right.norm();

        let mut toes = self.current_pose.toes;
        toes[swing_foot] = toe;

        let mut feet = self.current_pose.feet;
        feet[swing_foot] = target;

        self.active_step = Some(StepAnimation {
            foot,
            intent,
            foot_start: start,
            foot_target: target,
            foot_control: control,
            body_bob_height: bob,
            toe_roll_max,
        });

        self.target_pose = Pose {
            body: V3::new([body_pos.x0(), height + body_height, body_pos.x1()]),
            head: V3::new([body_pos.x0(), height + head_height, body_pos.x1()]),
            feet,
            toes,
            toe_dirs,
        };
    }

    pub fn finish_step(&mut self, keep_walking: bool) -> StepResult {
        match (self.state, &self.active_step, keep_walking) {
            // Continue walking → alternate foot
            (AnimationState::Stepping, Some(step), true) => StepResult::Advance(step.foot.other()),

            // Stop walking → close stance with trailing foot
            (AnimationState::Stepping, Some(step), false) => StepResult::Close(step.foot.other()),

            // Closing step finished → fully idle
            (AnimationState::Closing, _, _) => StepResult::Idle,

            _ => StepResult::Idle,
        }
    }

    pub fn position(&self) -> V4 {
        let pos = self.current_pose.body;
        V4::new([pos.x0(), pos.x1(), pos.x2(), 1.0])
    }

    pub fn update_debug_arrows(&mut self, context: &mut RenderContext) -> Result<()> {
        use crate::core::gl_pipeline_colored::arrow;

        for i in 0..2 {
            let foot_pos = self.current_pose.feet[i];
            let forward = self.current_pose.toe_dirs[i];
            let arrow_verts = arrow(foot_pos, forward, 1.5);
            context.update_colored_mesh(self.debug_arrows[i].mesh_id, &arrow_verts, &[])?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Component for Player {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        const TURN_SPEED: f32 = 1.5;
        let dt = ctx.dt_secs();
        self.phase_progress += dt;

        let move_forward = ctx.state.is_pressed(GameKey::MoveForward);
        if ctx.state.is_pressed(GameKey::StrafeLeft) {
            self.rotation -= TURN_SPEED * dt;
        }
        if ctx.state.is_pressed(GameKey::StrafeRight) {
            self.rotation += TURN_SPEED * dt;
        }

        let mut phase = self.phase_progress * self.step_speed;
        if phase >= 1.0 {
            phase = 0.0;

            let res = self.finish_step(move_forward);
            match res {
                StepResult::Idle => {
                    self.state = AnimationState::Idle;
                    self.active_step = None;
                    self.idle();
                }

                StepResult::Advance(foot) => {
                    self.state = AnimationState::Stepping;
                    self.step(ctx, foot, StepIntent::Advance);
                }

                StepResult::Close(foot) => {
                    self.state = AnimationState::Closing;
                    self.step(ctx, foot, StepIntent::Close);
                }
            }
        }

        if self.state == AnimationState::Idle && move_forward {
            self.state = AnimationState::Stepping;
            self.step(ctx, Foot::Left, StepIntent::Advance);
            phase = 0.0;
        }

        let mut feet_rot = [0.0, 0.0];
        match self.state {
            AnimationState::Idle => {
                self.current_pose = self.target_pose.clone();
            }
            AnimationState::Stepping | AnimationState::Closing => {
                let t = phase.clamp(0.0, 1.0);
                let mut pose = self.start_pose.lerp(&self.target_pose, t);

                if let Some(step) = &self.active_step {
                    let idx = step.foot.index_self();
                    pose.feet[idx] =
                        bezier_quad(step.foot_start, step.foot_control, step.foot_target, t);

                    feet_rot[idx] = step.toe_roll_max * toe_roll(t);

                    let bob = step.body_bob_height * body_bob(t);
                    pose.body += V3::new([0.0, bob, 0.0]);
                    pose.head += V3::new([0.0, bob * 0.8, 0.0]); // slight damping looks natural                
                }

                self.current_pose = pose;
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
        let rotation = Rotation::Euler(V3::new([0.0, rotation, 0.0]));
        self.objects[0].transform.rotation = rotation;
        self.objects[1].transform.rotation = rotation;

        let rotation = self.current_pose.toes[0].as_mat4x4() * affine4x4::rotate_x0(feet_rot[0]);
        self.objects[2].transform.rotation = Rotation::Matrix(rotation);

        let rotation = self.current_pose.toes[1].as_mat4x4() * affine4x4::rotate_x0(feet_rot[1]);
        self.objects[3].transform.rotation = Rotation::Matrix(rotation);

        Ok(())
    }
}
