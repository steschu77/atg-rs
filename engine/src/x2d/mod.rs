use crate::v2d::v2::V2;
pub mod circle;
pub mod collide;
pub mod manifold;
pub mod polygon;
pub mod rigid_body;

// ----------------------------------------------------------------------------
pub struct Mass {
    mass: f32,
    inertia: f32,
    inv_mass: f32,
    inv_inertia: f32,
}

impl Mass {
    // ------------------------------------------------------------------------
    pub fn new(mass: f32, inertia: f32) -> Self {
        Self {
            mass,
            inertia,
            inv_mass: 1.0 / mass,
            inv_inertia: 1.0 / inertia,
        }
    }

    // ------------------------------------------------------------------------
    pub fn from_radius(density: f32, radius: f32) -> Self {
        let area = std::f32::consts::PI * radius * radius;
        let mass = density * area;
        let inertia = mass * radius * radius / 2.0;
        Self::new(mass, inertia)
    }

    // ------------------------------------------------------------------------
    pub fn from_box(density: f32, w: &V2) -> Self {
        let area = w.x0() * w.x1();
        let area2 = w.x0() * w.x0() + w.x1() * w.x1();
        let mass = density * area;
        let inertia = mass * area2 / 12.0;
        Self::new(mass, inertia)
    }

    // ------------------------------------------------------------------------
    pub fn mass(&self) -> f32 {
        self.mass
    }

    // ------------------------------------------------------------------------
    pub fn inertia(&self) -> f32 {
        self.inertia
    }

    // ------------------------------------------------------------------------
    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }

    // ------------------------------------------------------------------------
    pub fn inv_inertia(&self) -> f32 {
        self.inv_inertia
    }
}

// ----------------------------------------------------------------------------
pub struct Material {
    density: f32,
    restitution: f32,
    static_friction: f32,
    dynamic_friction: f32,
}
