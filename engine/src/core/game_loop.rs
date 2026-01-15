use crate::core::{IClock, IGame, input};
use crate::error::Result;

pub struct GameLoop {
    dt_update: std::time::Duration,
    t_lag: std::time::Duration,
}

impl GameLoop {
    pub fn new(dt_update: std::time::Duration) -> Self {
        Self {
            dt_update,
            t_lag: std::time::Duration::ZERO,
        }
    }
    // ----------------------------------------------------------------------------
    pub fn step<Game: IGame, Clock: IClock>(
        &mut self,
        game: &mut Game,
        clock: &Clock,
        input: &mut input::Input,
    ) -> Result<()> {
        // game loop: https://gameprogrammingpatterns.com/game-loop.html
        let t0 = clock.now();

        // Slow machines: Clamp number of updates to avoid spiral of death
        // (otherwise the next loop will be late again)
        let updates_needed = (self.t_lag.as_nanos() / self.dt_update.as_nanos()) as u32 + 1;
        for _ in 0..updates_needed.min(4) {
            game.update(&self.dt_update, input)?;
        }

        game.render()?;

        let t1 = clock.now();
        self.t_lag += t1 - t0;

        if self.t_lag < self.dt_update {
            // Fast machines: sleep to maintain a consistent update rate
            clock.sleep(self.dt_update - self.t_lag);
        }

        // Pretend that all updates have been processed
        self.t_lag = self.t_lag.saturating_sub(self.dt_update * updates_needed);
        Ok(())
    }
}

// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tests::MockClock;
    use crate::core::tests::MockGame;

    #[test]
    fn test_gameloop_fast() {
        let t_step = std::time::Duration::from_millis(20);
        let t_update = std::time::Duration::from_millis(0);
        let t_render = std::time::Duration::from_millis(0);

        let mut input = input::Input::new();
        let clock = MockClock::default();
        let mut game = MockGame::new(&clock, t_update, t_render);
        let mut game_loop = GameLoop::new(t_step);
        for _ in 0..4 {
            let _ = game_loop.step(&mut game, &clock, &mut input);
        }

        // since processing time was 0 ms, every sleep call should be t_step
        assert_eq!(clock.sleeps(), vec![t_step; 4]);

        // since processing time was 0 ms, every loop should only contain one update
        assert_eq!(game.loops(), &vec![1; 4]);
    }

    #[test]
    fn test_gameloop_slow() {
        let t_step = std::time::Duration::from_millis(20);
        let t_update = std::time::Duration::from_millis(10);
        let t_render = std::time::Duration::from_millis(20);

        let mut input = input::Input::new();
        let clock = MockClock::default();
        let mut game = MockGame::new(&clock, t_update, t_render);
        let mut game_loop = GameLoop::new(t_step);
        for _ in 0..6 {
            let _ = game_loop.step(&mut game, &clock, &mut input);
        }

        // since updating time and rendering time are larger than loop time,
        // there should be no sleep
        assert_eq!(clock.sleeps(), vec![]);

        // with 10 ms updating time and 20 ms rendering time, we expect 2 updates per loop
        // give 2 loops to account for adoption time
        assert_eq!(game.loops()[2..6], vec![2; 4]);
    }

    #[test]
    fn test_gameloop_superslow() {
        let t_step = std::time::Duration::from_millis(20);
        let t_update = std::time::Duration::from_millis(20);
        let t_render = std::time::Duration::from_millis(20);

        let mut input = input::Input::new();
        let clock = MockClock::default();
        let mut game = MockGame::new(&clock, t_update, t_render);
        let mut game_loop = GameLoop::new(t_step);
        for _ in 0..6 {
            let _ = game_loop.step(&mut game, &clock, &mut input);
        }

        // since updating time and rendering time are larger than step time,
        // there should be no sleep
        assert_eq!(clock.sleeps(), vec![]);

        // with 20 ms updating time and 20 ms rendering time, we expect the maximum of 4 updates
        // per loop, give 3 loops to account for adoption time
        assert_eq!(game.loops()[3..6], vec![4; 3]);
    }
}
