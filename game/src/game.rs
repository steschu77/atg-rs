use engine::core::gl_renderer::Renderer;
use engine::core::world::World;
use engine::core::{IGame, IRenderer, input};
use engine::error::{Error, Result};
use engine::sys::opengl as gl;
use std::rc::Rc;

pub struct Game {
    renderer: Renderer,
    world: World,
}

impl IGame for Game {
    fn input(&mut self, events: &input::Events) -> Result<()> {
        self.world.input(events)?;
        self.input_events(events)
    }

    fn update(&mut self, dt: &std::time::Duration, state: &input::State) -> Result<()> {
        self.world.update(dt, state)?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let render_context = self.world.render_context();
        let camera = self.world.camera();
        let objects = self.world.objects();
        self.renderer.render(camera, objects, render_context)?;
        Ok(())
    }
}

impl Game {
    pub fn new(gl: gl::OpenGlFunctions) -> Result<Self> {
        let gl = Rc::new(gl);
        let renderer = Renderer::new(Rc::clone(&gl))?;
        let world = World::new(Rc::clone(&gl))?;
        Ok(Self { renderer, world })
    }

    pub fn resize(&mut self, cx: i32, cy: i32) {
        self.renderer.resize(cx, cy);
    }

    fn input_events(&mut self, events: &input::Events) -> Result<()> {
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
