use crate::core::{IClock, IGame, input};
use crate::error::Result;

pub struct GameLoop {
    dt_update: std::time::Duration,
    t_lag: std::time::Duration,
    t_prev: std::time::Duration,
}

impl GameLoop {
    pub fn new(dt_update: std::time::Duration) -> Self {
        Self {
            dt_update,
            t_lag: std::time::Duration::ZERO,
            t_prev: std::time::Duration::ZERO,
        }
    }
    // ----------------------------------------------------------------------------
    pub fn step<Game: IGame, Clock: IClock>(
        &mut self,
        game: &mut Game,
        clock: &Clock,
        events: &input::Events,
        state: &input::State,
    ) -> Result<()> {
        // game loop: https://gameprogrammingpatterns.com/game-loop.html
        let t_current = clock.now();
        let t_frame_total = t_current - self.t_prev;
        self.t_lag += t_frame_total;
        self.t_prev = t_current;

        game.input(events.clone(), state.clone())?;

        let updates_needed = (self.t_lag.as_nanos() / self.dt_update.as_nanos()) as u32;
        let updates_needed = updates_needed.max(1);

        // On slow machines we deliberately drop updates rather than spiral to death.
        // We accept simulation slowdown over instability.
        const MAX_UPDATES_PER_FRAME: u32 = 4;
        let updates_to_run = updates_needed.min(MAX_UPDATES_PER_FRAME);
        let updates_dropped = updates_needed - updates_to_run;

        for _ in 0..updates_to_run {
            game.update(&self.dt_update)?;
        }

        game.render()?;

        // Pretend that all updates have been processed. We are intentionally
        // forgetting the debt rather than carrying it forward.
        self.t_lag = self.t_lag.saturating_sub(self.dt_update * updates_needed);

        if updates_dropped > 0 {
            log::warn!("dropped {updates_dropped} update(s), lag={:?}", self.t_lag);
        }

        // Sleep for the remainder of the frame budget.
        let t_work = clock.t_since(t_current);
        let t_sleep = self.dt_update.saturating_sub(self.t_lag + t_work);
        if !t_sleep.is_zero() {
            clock.sleep(t_sleep);
        }

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

        let events = input::Events::default();
        let state = input::State::default();
        let clock = MockClock::default();
        let mut game = MockGame::new(&clock, t_update, t_render);
        let mut game_loop = GameLoop::new(t_step);
        for _ in 0..4 {
            let _ = game_loop.step(&mut game, &clock, &events, &state);
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

        let events = input::Events::default();
        let state = input::State::default();
        let clock = MockClock::default();
        let mut game = MockGame::new(&clock, t_update, t_render);
        let mut game_loop = GameLoop::new(t_step);
        for _ in 0..6 {
            let _ = game_loop.step(&mut game, &clock, &events, &state);
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

        let events = input::Events::default();
        let state = input::State::default();
        let clock = MockClock::default();
        let mut game = MockGame::new(&clock, t_update, t_render);
        let mut game_loop = GameLoop::new(t_step);
        for _ in 0..6 {
            let _ = game_loop.step(&mut game, &clock, &events, &state);
        }

        // since updating time and rendering time are larger than step time,
        // there should be no sleep
        assert_eq!(clock.sleeps(), vec![]);

        // with 20 ms updating time and 20 ms rendering time, we expect the maximum of 4 updates
        // per loop, give 3 loops to account for adoption time
        assert_eq!(game.loops()[3..6], vec![4; 3]);
    }
}
