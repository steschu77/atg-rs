use crate::v2d::r2::R2;
use crate::v2d::v2::V2;

// ----------------------------------------------------------------------------
pub struct Polygon {
    verts: [V2; 5],
    norms: [V2; 5],
    count: u32,
}

impl Polygon {
    // ------------------------------------------------------------------------
    pub fn new_poly3(p0: &V2, p1: &V2, p2: &V2) -> Self {
        let n0 = V2::normal(p0, p1);
        let n1 = V2::normal(p1, p2);
        let n2 = V2::normal(p2, p0);
        Self {
            verts: [*p0, *p1, *p2, V2::zero(), V2::zero()],
            norms: [n0, n1, n2, V2::zero(), V2::zero()],
            count: 3,
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_poly4(p0: &V2, p1: &V2, p2: &V2, p3: &V2) -> Self {
        let n0 = V2::normal(p0, p1);
        let n1 = V2::normal(p1, p2);
        let n2 = V2::normal(p2, p3);
        let n3 = V2::normal(p3, p0);
        Self {
            verts: [*p0, *p1, *p2, *p3, V2::zero()],
            norms: [n0, n1, n2, n3, V2::zero()],
            count: 4,
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_poly5(p0: &V2, p1: &V2, p2: &V2, p3: &V2, p4: &V2) -> Self {
        let n0 = V2::normal(p0, p1);
        let n1 = V2::normal(p1, p2);
        let n2 = V2::normal(p2, p3);
        let n3 = V2::normal(p3, p4);
        let n4 = V2::normal(p4, p0);
        Self {
            verts: [*p0, *p1, *p2, *p3, *p4],
            norms: [n0, n1, n2, n3, n4],
            count: 5,
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_box(w: &V2) -> Self {
        let h = 0.5 * w;
        let n0 = V2::new([-1.0, 0.0]);
        let n1 = V2::new([0.0, -1.0]);
        let n2 = V2::new([1.0, 0.0]);
        let n3 = V2::new([0.0, 1.0]);
        Self {
            verts: [
                V2::new([-h.x0(), -h.x1()]),
                V2::new([h.x0(), -h.x1()]),
                V2::new([h.x0(), h.x1()]),
                V2::new([-h.x0(), h.x1()]),
                V2::zero(),
            ],
            norms: [n0, n1, n2, n3, V2::zero()],
            count: 4,
        }
    }

    // ------------------------------------------------------------------------
    pub fn new_circle(radius: f32, segments: u32) -> Self {
        let mut s = Polygon {
            verts: [V2::zero(); 5],
            norms: [V2::zero(); 5],
            count: segments,
        };
        let mut angle = 0.0;
        let da = 2.0 * std::f32::consts::PI / segments as f32;
        for i in 0..segments as usize {
            let r = R2::new(angle);
            s.verts[i] = radius * r.x_axis();
            s.norms[i] = r.x_axis();
            angle += da;
        }
        s
    }

    // ------------------------------------------------------------------------
    pub fn count(&self) -> u32 {
        self.count
    }

    // ------------------------------------------------------------------------
    pub fn verts(&self) -> &[V2] {
        &self.verts[0..self.count as usize]
    }

    // ------------------------------------------------------------------------
    pub fn norms(&self) -> &[V2] {
        &self.norms[0..self.count as usize]
    }

    // ------------------------------------------------------------------------
    pub fn xform(&self, pos: &V2, angle: f32) -> Self {
        let mut s = Polygon {
            verts: [V2::zero(); 5],
            norms: [V2::zero(); 5],
            count: self.count,
        };
        let q = R2::new(angle);
        for i in 0..self.count as usize {
            s.verts[i] = q * self.verts[i] + *pos;
        }
        for i in 0..self.count as usize {
            s.norms[i] = q * self.norms[i];
        }
        s
    }
}
