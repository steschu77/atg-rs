use crate::error::Result;

pub mod camera;
pub mod clock;
pub mod component;
pub mod game_loop;
pub mod gl_font;
pub mod gl_graphics;
pub mod gl_pipeline;
pub mod gl_pipeline_colored;
pub mod gl_pipeline_msdftex;
pub mod gl_renderer;
pub mod gl_text;
pub mod gl_texture;
pub mod input;
pub mod player;
pub mod terrain;
pub mod world;

// ----------------------------------------------------------------------------
pub trait IClock {
    fn now(&self) -> std::time::Duration;
    fn sleep(&self, dt: std::time::Duration) -> std::time::Duration;
}

// ----------------------------------------------------------------------------
pub trait IGame {
    fn input(&mut self, events: &input::Events) -> Result<()>;
    fn update(&mut self, dt: &std::time::Duration, state: &input::State) -> Result<()>;
    fn render(&mut self) -> Result<()>;
}

// ----------------------------------------------------------------------------
pub trait IRenderer {
    fn render(
        &self,
        camera: &camera::Camera,
        objects: Vec<gl_renderer::RenderObject>,
        context: &gl_renderer::RenderContext,
    ) -> Result<()>;
    fn resize(&self, cx: i32, cy: i32);
}

// ----------------------------------------------------------------------------
#[cfg(test)]
pub mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;

    pub struct MockClock {
        t: Cell<std::time::Duration>,
        sleeps: RefCell<Vec<std::time::Duration>>,
    }

    impl IClock for MockClock {
        fn now(&self) -> std::time::Duration {
            self.t.get()
        }

        fn sleep(&self, dt: std::time::Duration) -> std::time::Duration {
            self.sleeps.borrow_mut().push(dt);
            self.t.set(self.t.get() + dt);
            self.t.get()
        }
    }

    impl Default for MockClock {
        fn default() -> Self {
            Self {
                t: Cell::new(std::time::Duration::from_secs(0)),
                sleeps: RefCell::new(Vec::new()),
            }
        }
    }

    impl MockClock {
        fn advance(&self, dt: std::time::Duration) -> std::time::Duration {
            self.t.set(self.t.get() + dt);
            self.t.get()
        }

        pub fn sleeps(&self) -> Vec<std::time::Duration> {
            self.sleeps.borrow().clone()
        }
    }

    pub struct MockGame<'a> {
        clock: &'a MockClock,
        t_update: std::time::Duration,
        t_render: std::time::Duration,
        update_count: usize,
        loops: Vec<usize>,
    }

    impl IGame for MockGame<'_> {
        fn input(&mut self, _events: &input::Events) -> Result<()> {
            Ok(())
        }

        fn update(&mut self, _dt: &std::time::Duration, _state: &input::State) -> Result<()> {
            self.update_count += 1;
            self.clock.advance(self.t_update);
            Ok(())
        }

        fn render(&mut self) -> Result<()> {
            self.loops.push(self.update_count);
            self.update_count = 0;
            self.clock.advance(self.t_render);
            Ok(())
        }
    }

    impl<'a> MockGame<'a> {
        pub fn new(
            clock: &'a MockClock,
            t_update: std::time::Duration,
            t_render: std::time::Duration,
        ) -> Self {
            Self {
                clock,
                t_update,
                t_render,
                update_count: 0,
                loops: Vec::new(),
            }
        }

        pub fn loops(&self) -> &Vec<usize> {
            &self.loops
        }
    }

    #[test]
    fn test_game() {
        let input = input::Input::new();
        let clock = MockClock::default();
        let mut game = MockGame::new(
            &clock,
            std::time::Duration::from_millis(10),
            std::time::Duration::from_millis(20),
        );
        assert_eq!(game.update(&clock.now(), &input.take_state()), Ok(()));
        assert_eq!(game.render(), Ok(()));
        assert_eq!(game.loops().len(), 1);
    }
}
