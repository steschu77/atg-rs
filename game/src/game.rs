use engine::core::gl_renderer::Renderer;
use engine::core::world::World;
use engine::core::{IGame, IRenderer, input};
use engine::error::{Error, Result};
use engine::sys::opengl as gl;

pub struct Game {
    renderer: Renderer,
    world: World,
}

impl IGame for Game {
    fn update(
        &mut self,
        dt: &std::time::Duration,
        events: &input::Events,
        state: &input::State,
    ) -> Result<()> {
        self.input_events(events)?;
        self.world.update(dt, events, state)?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.renderer.render(&self.world)?;
        Ok(())
    }
}

impl Game {
    pub fn new(gl: gl::OpenGlFunctions) -> Result<Self> {
        Ok(Self {
            renderer: Renderer::new(gl)?,
            world: World::default(),
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
                _ => {}
            }
        }
        Ok(())
    }
}
