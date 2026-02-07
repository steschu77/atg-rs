pub mod mass;
pub mod rigid_body;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    density: f32,
    restitution: f32,
    static_friction: f32,
    dynamic_friction: f32,
}
