use engine::core::gl_renderer::Renderer;
use engine::core::world::World;
use engine::core::{IGame, IRenderer, input};
use engine::error::{Error, Result};
use engine::sys::opengl as gl;

pub struct Game {
    renderer: Renderer,
    world: World,
    t_update: std::time::Duration,
}

impl IGame for Game {
    fn update(&mut self, t_now: &std::time::Duration, input: &mut input::Input) -> Result<()> {
        // Update the game world, e.g., physics, AI, etc.
        self.world.update(t_now)?;

        let events = input.take_events();
        self.input_events(&events)?;

        self.input_state(input);
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        // Update the renderer with the current state of the world
        self.renderer.render(&self.world)?;
        Ok(())
    }
}

impl Game {
    pub fn new(gl: gl::OpenGlFunctions, t_update: std::time::Duration) -> Result<Self> {
        Ok(Self {
            renderer: Renderer::new(gl)?,
            world: World::default(),
            t_update,
        })
    }

    pub fn resize(&mut self, cx: i32, cy: i32) {
        self.renderer.resize(cx, cy);
    }

    fn input_events(&mut self, events: &[input::Event]) -> Result<()> {
        // Process input events, e.g., keyboard, mouse, etc.
        for event in events {
            match event {
                input::Event::KeyUp {
                    key: input::Key::Exit,
                } => {
                    return Err(Error::GameOver);
                }
                input::Event::ButtonUp { button: 3 } => {
                    return Err(Error::GameOver);
                }
                input::Event::MouseMove { x, y } => {
                    self.world.pan_camera(*x as f32 * 0.01);
                    self.world.tilt_camera(*y as f32 * 0.01);
                    //let d = V4::new([*x as f32, *y as f32, 0.0, 0.0]);
                    //self.world.move_camera(d);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn input_state(&mut self, input: &input::Input) {
        // Update the game state based on the current input state
        if input.is_pressed(input::Key::MoveForward) {
            self.world.move_forward(4.0 * self.t_update.as_secs_f32());
        }
        if input.is_pressed(input::Key::MoveBackward) {
            self.world.move_backward(4.0 * self.t_update.as_secs_f32());
        }
        if input.is_pressed(input::Key::StrafeLeft) {
            self.world.strafe_left(4.0 * self.t_update.as_secs_f32());
        }
        if input.is_pressed(input::Key::StrafeRight) {
            self.world.strafe_right(4.0 * self.t_update.as_secs_f32());
        }
    }
}
