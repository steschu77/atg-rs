use crate::error::{Error, Result};
use crate::v2d::Positive;
use crate::v2d::v3::V3;

// ----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct Mass {
    mass: f32,
    inertia: V3,
    inv_mass: f32,
    inv_inertia: V3,
}

impl Mass {
    // ------------------------------------------------------------------------
    pub fn new(mass: f32, inertia: V3) -> Result<Self> {
        if !mass.is_positive() || !inertia.is_positive() {
            Err(Error::InvalidData)
        } else {
            Ok(Self::build_v3(mass, inertia))
        }
    }

    // ------------------------------------------------------------------------
    pub fn from_sphere(density: f32, radius: f32) -> Result<Self> {
        if !density.is_positive() || !radius.is_positive() {
            return Err(Error::InvalidData);
        }
        let volume = (4.0 / 3.0) * std::f32::consts::PI * radius * radius * radius;
        let mass = density * volume;
        let inertia = mass * radius * radius * 0.4; // 2/5 * m * r^2
        Ok(Self::build_scalar(mass, inertia))
    }

    // ------------------------------------------------------------------------
    pub fn from_box(density: f32, w: V3) -> Result<Self> {
        if !density.is_positive() || !w.is_positive() {
            return Err(Error::InvalidData);
        }
        let volume = w.x0() * w.x1() * w.x2();
        let mass = density * volume;
        let inertia = mass / 12.0
            * V3::new([
                (w.x1() * w.x1() + w.x2() * w.x2()),
                (w.x2() * w.x2() + w.x0() * w.x0()),
                (w.x0() * w.x0() + w.x1() * w.x1()),
            ]);
        Ok(Self::build_v3(mass, inertia))
    }

    // ------------------------------------------------------------------------
    pub fn mass(&self) -> f32 {
        self.mass
    }

    // ------------------------------------------------------------------------
    pub fn inertia(&self) -> V3 {
        self.inertia
    }

    // ------------------------------------------------------------------------
    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }

    // ------------------------------------------------------------------------
    pub fn inv_inertia(&self) -> V3 {
        self.inv_inertia
    }

    // ------------------------------------------------------------------------
    fn build_scalar(mass: f32, inertia: f32) -> Self {
        debug_assert!(mass.is_positive());
        debug_assert!(inertia.is_positive());

        Self {
            mass,
            inertia: V3::uniform(inertia),
            inv_mass: 1.0 / mass,
            inv_inertia: V3::uniform(1.0 / inertia),
        }
    }

    // ------------------------------------------------------------------------
    fn build_v3(mass: f32, inertia: V3) -> Self {
        debug_assert!(mass.is_positive());
        debug_assert!(inertia.is_positive());

        Self {
            mass,
            inertia,
            inv_mass: 1.0 / mass,
            inv_inertia: 1.0 / inertia,
        }
    }
}

// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_float_eq;

    // ------------------------------------------------------------------------
    #[test]
    fn new_valid_mass() {
        let m = Mass::new(2.0, V3::new([3.0, 4.0, 5.0])).unwrap();

        assert_float_eq!(m.mass(), 2.0);
        assert_float_eq!(m.inv_mass(), 0.5);

        let i = m.inertia();
        let ii = m.inv_inertia();
        assert_float_eq!(i.x0() * ii.x0(), 1.0);
        assert_float_eq!(i.x1() * ii.x1(), 1.0);
        assert_float_eq!(i.x2() * ii.x2(), 1.0);
    }

    // ------------------------------------------------------------------------
    #[test]
    fn new_invalid_mass() {
        assert!(Mass::new(0.0, V3::uniform(1.0)).is_err());
        assert!(Mass::new(1.0, V3::new([0.0, 1.0, 1.0])).is_err());
    }

    // ------------------------------------------------------------------------
    #[test]
    fn sphere_mass_properties() {
        let density = 1.0 / ((4.0 / 3.0) * std::f32::consts::PI); // mass = 1.0 for radius = 1.0
        let radius = 1.0;

        let m = Mass::from_sphere(density, radius).unwrap();

        assert_float_eq!(m.mass(), 1.0);
        assert_float_eq!(m.inertia().x0(), 0.4);
        assert_float_eq!(m.inertia().x1(), 0.4);
        assert_float_eq!(m.inertia().x2(), 0.4);
    }

    // ------------------------------------------------------------------------
    #[test]
    fn box_mass_properties() {
        let density = 1.0;
        let w = V3::new([0.5, 1.0, 2.0]); // 1.0 m³
        let m = Mass::from_box(density, w).unwrap();

        assert_float_eq!(m.mass(), 1.0);

        let i = m.inertia();
        assert_float_eq!(i.x0(), 1.0 / 12.0 * (1.0 * 1.0 + 2.0 * 2.0));
        assert_float_eq!(i.x1(), 1.0 / 12.0 * (2.0 * 2.0 + 0.5 * 0.5));
        assert_float_eq!(i.x2(), 1.0 / 12.0 * (0.5 * 0.5 + 1.0 * 1.0));
    }
}
