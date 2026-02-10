pub mod mass;
pub mod rigid_body;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, Default)]
pub struct Material {
    pub density: f32,
    pub restitution: f32,
    pub static_friction: f32,
    pub dynamic_friction: f32,
}

// ----------------------------------------------------------------------------
pub const WOOD: Material = Material {
    density: 700.0,
    restitution: 0.5,
    static_friction: 0.4,
    dynamic_friction: 0.3,
};

// ----------------------------------------------------------------------------
pub const STEEL: Material = Material {
    density: 7850.0,
    restitution: 0.1,
    static_friction: 0.6,
    dynamic_friction: 0.5,
};

// ----------------------------------------------------------------------------
pub const RUBBER: Material = Material {
    density: 1500.0,
    restitution: 0.8,
    static_friction: 1.0,
    dynamic_friction: 0.8,
};
