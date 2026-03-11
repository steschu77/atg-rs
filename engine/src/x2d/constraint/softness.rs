// ----------------------------------------------------------------------------
#[derive(Debug, Default, Clone)]
pub struct Softness {
    pub bias_rate: f32,
    pub mass_scale: f32,
    pub impulse_scale: f32,
}

// ----------------------------------------------------------------------------
impl Softness {
    pub fn new(hertz: f32, zeta: f32, h: f32) -> Self {
        if hertz == 0.0 {
            return Softness::default();
        }

        let omega = std::f32::consts::TAU * hertz;

        let a1 = 2.0 * zeta + h * omega;
        let a2 = h * omega * a1;
        let a3 = 1.0 / (1.0 + a2);

        Softness {
            bias_rate: omega / a1,
            mass_scale: a2 * a3,
            impulse_scale: a3,
        }
    }
}
