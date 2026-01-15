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
}

// ----------------------------------------------------------------------------
impl Camera {
    pub fn new(position: V4, direction: V4) -> Self {
        Self {
            position,
            direction,
            speed: V4::new([0.0, 0.0, 0.0, 0.0]),
        }
    }

    pub fn update(&mut self, dt: &std::time::Duration) -> Result<()> {
        self.position += self.speed * dt.as_secs_f32();
        Ok(())
    }

    pub fn position(&self) -> V4 {
        self.position
    }

    pub fn direction(&self) -> V4 {
        self.direction
    }

    pub fn transform(&self) -> M4x4 {
        /*let ogl_coordinate_system = M4x4::zero()
            .with((0, 0), 1.0)
            .with((2, 1), 1.0)
            .with((1, 2), 1.0)
            .with((3, 3), 1.0);
        ogl_coordinate_system
            * */
        let front = V4::new([0.0, 0.0, -1.0, 0.0]);
        let front = affine4x4::rotate_x0(self.direction.x0()) * front;
        let front = affine4x4::rotate_x1(self.direction.x1()) * front;
        affine4x4::look_at(
            self.position,
            self.position + front,
            V4::new([0.0, 1.0, 0.0, 0.0]),
        )
    }

    // pub fn transform(&self) -> M4x4 {
    //     let view = affine4x4::look_at(
    //         camera.position(),
    //         camera.target(),
    //         V4::new([0.0, 0.0, 1.0, 0.0]),
    //     );
    // }

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
