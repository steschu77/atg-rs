use crate::v2d::v2::V2;

// ----------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
struct Circle {
    center: V2,
    r: f32,
    r2: f32,
}

impl Circle {
    // ------------------------------------------------------------------------
    pub fn new(center: &V2, r: f32) -> Self {
        Self {
            center: *center,
            r,
            r2: r * r,
        }
    }

    // ------------------------------------------------------------------------
    // Find the center and radius for the circle through p0, p1, p2.
    pub fn from_points(p0: &V2, p1: &V2, p2: &V2) -> Self {
        let ab = *p1 - *p0;
        let ac = *p2 - *p0;
        let bc = *p2 - *p1;

        let ab2 = *p1 + *p0;
        let ac2 = *p2 + *p0;

        // Algorithm from O'Rourke 2ed p. 189.
        let a = ab.x0();
        let b = ab.x1();
        let c = ac.x0();
        let d = ac.x1();
        let e = ab.x0() * ab2.x0() + ab.x1() * ab2.x1();
        let f = ac.x0() * ac2.x0() + ac.x1() * ac2.x1();
        let g = 2.0 * (ab.x0() * bc.x1() - ab.x1() * bc.x0());

        if g.abs() < 0.0001 {
            return Self::default(); // Points are co-linear.
        }

        // Point o is the center of the circle.
        let o = V2::new([(d * e - b * f) / g, (a * f - c * e) / g]);
        let r2 = (*p0 - o).length2();

        Self {
            center: o,
            r: r2.sqrt(),
            r2,
        }
    }

    // ------------------------------------------------------------------------
    pub fn contains(&self, pos: &V2) -> bool {
        let d = *pos - self.center;
        d.length2() <= self.r2
    }

    // ------------------------------------------------------------------------
    pub fn xform(&self, pos: &V2) -> Self {
        Self {
            center: self.center + *pos,
            r: self.r,
            r2: self.r2,
        }
    }
}
