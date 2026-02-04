use crate::core::game_input;
use crate::core::terrain;
use crate::error::Result;
use std::time::Duration;

// ----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Context<'a> {
    pub dt: Duration,
    pub state: &'a game_input::InputContext,
    pub terrain: &'a terrain::Terrain,
}

// ----------------------------------------------------------------------------
impl<'a> Context<'a> {
    pub fn dt_secs(&self) -> f32 {
        self.dt.as_secs_f32()
    }
}

// ----------------------------------------------------------------------------
pub trait Component {
    fn update(&mut self, ctx: &Context) -> Result<()>;
}
