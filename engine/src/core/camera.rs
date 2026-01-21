use crate::core::component::{Component, Context};
use crate::core::input;
use crate::error::Result;
use crate::v2d::{affine4x4, m4x4::M4x4, v4::V4};

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Camera {
    position: V4,
    direction: V4,
    velocity: V4,
    target: V4,
    target_forward: V4,
    target_smoothed: V4,
    distance: f32,
    stiffness: f32,
    damping: f32,
}

// ----------------------------------------------------------------------------
impl Component for Camera {
    fn update(&mut self, ctx: &Context) -> Result<()> {
        let dt = ctx.dt_secs();

        // Smoothing the target position
        let d = self.target_smoothed - self.target;
        let accel = -self.stiffness * d - self.damping * self.velocity;
        self.velocity += accel * dt;
        self.target_smoothed += self.velocity * dt;

        // Responsive camera rotation
        let yaw = affine4x4::rotate_x1(self.direction.x1());
        let offset = yaw * (-self.target_forward.norm() * self.distance);

        // Adapt height based on terrain
        let position = self.target_smoothed + offset + V4::new([0.0, 4.0, 0.0, 0.0]);
        let height = ctx.terrain.height_at(position.x0(), position.x2());
        let target_x1 = position.x1().max(height + 1.0);

        self.position = V4::new([position.x0(), target_x1, position.x2(), 1.0]);
        Ok(())
    }
}

// ----------------------------------------------------------------------------
impl Camera {
    pub fn new(position: V4, direction: V4) -> Self {
        let target = V4::new([0.0, 0.0, -1.0, 0.0]);
        Self {
            position,
            direction,
            velocity: V4::new([0.0, 0.0, 0.0, 0.0]),
            target,
            target_forward: V4::new([0.0, 0.0, -1.0, 0.0]),
            target_smoothed: target,
            distance: 4.0,
            stiffness: 50.0,
            damping: 10.0,
        }
    }

    pub fn position(&self) -> V4 {
        self.position
    }

    pub fn input(&mut self, events: &input::Events) -> Result<()> {
        // Process input events, e.g., keyboard, mouse, etc.
        for event in events {
            #[allow(clippy::single_match)]
            match event {
                input::Event::MouseMove { x, y } => {
                    self.yaw(*x as f32 * 0.01);
                    self.tilt(*y as f32 * 0.01);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn transform(&self) -> M4x4 {
        let pitch = affine4x4::rotate_x0(-self.direction.x0());
        let look_at = affine4x4::look_at(self.position, self.target, V4::new([0.0, 1.0, 0.0, 0.0]));
        pitch * look_at
    }

    pub fn look_at(&mut self, target: V4, forward: V4) {
        self.target = target;
        self.target_forward = forward;
    }

    fn move_by(&mut self, d: V4) {
        let transform = self.transform().inverse();
        self.position += transform * d;
    }

    pub fn move_forward(&mut self, distance: f32) {
        self.move_by(V4::new([0.0, 0.0, -distance, 0.0]));
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.move_by(V4::new([0.0, 0.0, distance, 0.0]));
    }

    pub fn strafe_left(&mut self, distance: f32) {
        self.move_by(V4::new([-distance, 0.0, 0.0, 0.0]));
    }

    pub fn strafe_right(&mut self, distance: f32) {
        self.move_by(V4::new([distance, 0.0, 0.0, 0.0]));
    }

    pub fn yaw(&mut self, x: f32) {
        self.direction += V4::new([0.0, x, 0.0, 0.0]);
    }

    pub fn tilt(&mut self, y: f32) {
        self.direction += V4::new([y, 0.0, 0.0, 0.0]);
    }
}
