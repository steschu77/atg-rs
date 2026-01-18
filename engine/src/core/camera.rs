use crate::core::input;
use crate::error::Result;
use crate::v2d::affine4x4;
use crate::v2d::m4x4::M4x4;
use crate::v2d::v4::V4;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Camera {
    position: V4,
    direction: V4,
    speed: V4,
    target: V4,
}

// ----------------------------------------------------------------------------
impl Camera {
    pub fn new(position: V4, direction: V4) -> Self {
        Self {
            position,
            direction,
            speed: V4::new([0.0, 0.0, 0.0, 0.0]),
            target: V4::new([0.0, 0.0, -1.0, 0.0]),
        }
    }

    pub fn update(&mut self, dt: &std::time::Duration, events: &input::Events) -> Result<()> {
        self.position += self.speed * dt.as_secs_f32();
        self.input_events(events)?;
        Ok(())
    }

    pub fn position(&self) -> V4 {
        self.position
    }

    pub fn direction(&self) -> V4 {
        self.direction
    }

    pub fn transform(&self) -> M4x4 {
        let look_at = affine4x4::look_at(self.position, self.target, V4::new([0.0, 1.0, 0.0, 0.0]));

        let rotate_x0 = affine4x4::rotate_x0(-self.direction.x0());
        let rotate_x1 = affine4x4::rotate_x1(-self.direction.x1());
        rotate_x0 * rotate_x1 * look_at
    }

    pub fn look_at(&mut self, target: V4) {
        self.target = target;
    }

    fn input_events(&mut self, events: &[input::Event]) -> Result<()> {
        // Process input events, e.g., keyboard, mouse, etc.
        for event in events {
            #[allow(clippy::single_match)]
            match event {
                input::Event::MouseMove { x, y } => {
                    self.pan(*x as f32 * 0.01);
                    self.tilt(*y as f32 * 0.01);
                }
                _ => {}
            }
        }
        Ok(())
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

    pub fn pan(&mut self, x: f32) {
        self.direction += V4::new([0.0, x, 0.0, 0.0]);
    }

    pub fn tilt(&mut self, y: f32) {
        self.direction += V4::new([y, 0.0, 0.0, 0.0]);
    }
}
