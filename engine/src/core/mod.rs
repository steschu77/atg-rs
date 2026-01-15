use crate::error::Result;

pub mod camera;
pub mod clock;
pub mod game_loop;
pub mod gl_graphics;
pub mod gl_pipeline;
pub mod gl_renderer;
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
    fn update(&mut self, t_now: &std::time::Duration, input: &mut input::Input) -> Result<()>;
    fn render(&mut self) -> Result<()>;
}

// ----------------------------------------------------------------------------
pub trait IRenderer {
    fn render(&self, world: &world::World) -> Result<()>;
    fn resize(&self, cx: i32, cy: i32);
}

// ----------------------------------------------------------------------------
pub trait IScene {}

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
        fn update(
            &mut self,
            _t_now: &std::time::Duration,
            _input: &mut input::Input,
        ) -> Result<()> {
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
        let mut input = input::Input::new();
        let clock = MockClock::default();
        let mut game = MockGame::new(
            &clock,
            std::time::Duration::from_millis(10),
            std::time::Duration::from_millis(20),
        );
        assert_eq!(game.update(&clock.now(), &mut input), Ok(()));
        assert_eq!(game.render(), Ok(()));
        assert_eq!(game.loops().len(), 1);
    }
}
