use crate::core::IClock;

// ----------------------------------------------------------------------------
pub struct Clock {
    t0: std::time::Instant,
}

// ----------------------------------------------------------------------------
impl IClock for Clock {
    fn now(&self) -> std::time::Duration {
        let t1 = std::time::Instant::now();
        t1.duration_since(self.t0)
    }

    fn sleep(&self, dt: std::time::Duration) -> std::time::Duration {
        std::thread::sleep(dt);
        self.now()
    }
}

// ----------------------------------------------------------------------------
impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
impl Clock {
    pub fn new() -> Self {
        Clock {
            t0: std::time::Instant::now(),
        }
    }
}
