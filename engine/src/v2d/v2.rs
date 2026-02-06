use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use super::Positive;
use super::float_eq::float_eq_rel;
use super::v3::V3;

// ----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct V2 {
    m: [f32; 2],
}

// ----------------------------------------------------------------------------
impl Default for V2 {
    fn default() -> Self {
        V2::zero()
    }
}

// ----------------------------------------------------------------------------
impl fmt::Display for V2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V2({:.2}, {:.2})", self.x0(), self.x1())
    }
}

// ----------------------------------------------------------------------------
impl PartialEq for V2 {
    #[rustfmt::skip]
    fn eq(&self, rhs: &Self) -> bool {
        float_eq_rel(self.x0(), rhs.x0()) &&
        float_eq_rel(self.x1(), rhs.x1())
    }
}

// ----------------------------------------------------------------------------
impl Positive for V2 {
    fn is_positive(&self) -> bool {
        self.x0().is_positive() && self.x1().is_positive()
    }
}

// ----------------------------------------------------------------------------
impl Add for V2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x0 = self.x0() + rhs.x0();
        let x1 = self.x1() + rhs.x1();
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
impl Sub for V2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x0 = self.x0() - rhs.x0();
        let x1 = self.x1() - rhs.x1();
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
// V2 * f32 -> V2
impl Mul<f32> for V2 {
    type Output = Self;

    fn mul(self, s: f32) -> Self {
        let x0 = self.x0() * s;
        let x1 = self.x1() * s;
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
// V2 / f32 -> V2
impl Div<f32> for V2 {
    type Output = Self;

    fn div(self, s: f32) -> Self {
        let inv_s = 1.0 / s;
        let x0 = self.x0() * inv_s;
        let x1 = self.x1() * inv_s;
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
// f32 * V2 -> V2
impl Mul<V2> for f32 {
    type Output = V2;

    fn mul(self, v: V2) -> V2 {
        let x0 = self * v.x0();
        let x1 = self * v.x1();
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
// f32 / V2 -> V2
impl Div<V2> for f32 {
    type Output = V2;

    fn div(self, v: V2) -> V2 {
        let x0 = self / v.x0();
        let x1 = self / v.x1();
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
// f32 * V2 -> V2
impl Mul<&V2> for f32 {
    type Output = V2;

    fn mul(self, v: &V2) -> V2 {
        let x0 = self * v.x0();
        let x1 = self * v.x1();
        V2::new([x0, x1])
    }
}

// ----------------------------------------------------------------------------
// V2 * V2 -> f32
impl Mul for V2 {
    type Output = f32;

    fn mul(self, rhs: Self) -> f32 {
        self.x0() * rhs.x0() + self.x1() * rhs.x1()
    }
}

// ----------------------------------------------------------------------------
impl Neg for V2 {
    type Output = Self;

    fn neg(self) -> Self {
        V2::new([-self.x0(), -self.x1()])
    }
}

// ----------------------------------------------------------------------------
impl AddAssign for V2 {
    fn add_assign(&mut self, rhs: Self) {
        self.m[0] += rhs.x0();
        self.m[1] += rhs.x1();
    }
}

// ----------------------------------------------------------------------------
impl SubAssign for V2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.m[0] -= rhs.x0();
        self.m[1] -= rhs.x1();
    }
}

// ----------------------------------------------------------------------------
impl MulAssign<f32> for V2 {
    fn mul_assign(&mut self, s: f32) {
        self.m[0] *= s;
        self.m[1] *= s;
    }
}

// ----------------------------------------------------------------------------
impl From<[f32; 2]> for V2 {
    fn from(m: [f32; 2]) -> Self {
        V2 { m }
    }
}

// ----------------------------------------------------------------------------
impl From<V3> for V2 {
    fn from(v: V3) -> Self {
        V2::new([v.x0(), v.x1()])
    }
}

// ----------------------------------------------------------------------------
impl V2 {
    // ------------------------------------------------------------------------
    pub const fn new(m: [f32; 2]) -> Self {
        V2 { m }
    }

    // ------------------------------------------------------------------------
    pub const fn zero() -> Self {
        V2::new([0.0, 0.0])
    }

    // ------------------------------------------------------------------------
    pub const fn from_slice(m: &[f32; 2]) -> Self {
        V2 { m: *m }
    }

    // ------------------------------------------------------------------------
    pub const fn with_x0(mut self, value: f32) -> Self {
        self.m[0] = value;
        self
    }

    // ------------------------------------------------------------------------
    pub const fn with_x1(mut self, value: f32) -> Self {
        self.m[1] = value;
        self
    }

    // ------------------------------------------------------------------------
    pub const X0: V2 = V2::new([1.0, 0.0]);
    pub const X1: V2 = V2::new([0.0, 1.0]);

    // ------------------------------------------------------------------------
    pub const fn x0(&self) -> f32 {
        self.m[0]
    }

    // ------------------------------------------------------------------------
    pub const fn x1(&self) -> f32 {
        self.m[1]
    }

    // ------------------------------------------------------------------------
    pub fn as_array(&self) -> [f32; 2] {
        self.m
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }

    // ------------------------------------------------------------------------
    pub const fn perpendicular(&self) -> Self {
        V2::new([-self.x1(), self.x0()])
    }

    // ------------------------------------------------------------------------
    pub const fn length2(&self) -> f32 {
        self.x0() * self.x0() + self.x1() * self.x1()
    }

    // ------------------------------------------------------------------------
    pub fn length(&self) -> f32 {
        self.length2().sqrt()
    }

    // ------------------------------------------------------------------------
    pub fn distance(x0: &Self, x1: &Self) -> f32 {
        let d = *x1 - *x0;
        d.length()
    }

    // ------------------------------------------------------------------------
    pub fn norm(&self) -> Self {
        let l2 = self.length2();
        if l2 < f32::EPSILON {
            V2::default()
        } else {
            let inv_l = 1.0 / l2.sqrt();
            let x0 = self.x0() * inv_l;
            let x1 = self.x1() * inv_l;
            V2::new([x0, x1])
        }
    }

    // ------------------------------------------------------------------------
    pub fn abs(&self) -> Self {
        V2::new([self.x0().abs(), self.x1().abs()])
    }

    // ------------------------------------------------------------------------
    pub const fn dot(v0: &Self, v1: &Self) -> f32 {
        v0.x0() * v1.x0() + v0.x1() * v1.x1()
    }

    // ----------------------------------------------------------------------------
    // Two "crossed" vectors return a scalar, which is:
    // * area of the parallelogram of the 2 vectors
    // * magnitude of the Z vector of 3D cross product
    // * signed and determines v0 rotates CW or CCW to v1 or v0 and v1 are co-linear
    // * determinant of the 2x2 matrix built from vectors v0 and v1
    pub const fn cross(v0: &Self, v1: &Self) -> f32 {
        v0.x0() * v1.x1() - v0.x1() * v1.x0()
    }

    // ----------------------------------------------------------------------------
    // k == 0: v0, v1, v2 triplet is co-linear
    // k >  0: v0, v1, v2 triplet is clockwise
    // k <  0: v0, v1, v2 triplet is counter clockwise
    pub fn winding(v0: &Self, v1: &Self, v2: &Self) -> f32 {
        Self::cross(&(*v0 - *v1), &(*v0 - *v2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2() {
        let v0 = V2::new([1.0, 2.0]);
        let v1 = V2::new([3.0, 4.0]);
        let v2 = v0 - v1;

        assert_eq!(v2.x0(), -2.0);
        assert_eq!(v2.x1(), -2.0);
        assert_eq!(v0 + v1, V2::new([4.0, 6.0]));
        assert_eq!(v0 * 2.0, V2::new([2.0, 4.0]));
        assert_eq!(2.0 * v0, V2::new([2.0, 4.0]));
        assert_eq!(v0 * v1, 11.0);
        assert_eq!(-v0, V2::new([-1.0, -2.0]));
        assert_eq!(v0.perpendicular(), V2::new([-2.0, 1.0]));
        assert_eq!(v0.length2(), 5.0);
        assert_eq!(v1.length(), 5.0);
        assert_eq!(v1.norm(), V2::new([0.6, 0.8]));
        assert_eq!(v2.abs(), V2::new([2.0, 2.0]));
        assert_eq!(V2::distance(&v0, &v2), 5.0);
        assert_eq!(V2::dot(&v0, &v1), 11.0);
        assert_eq!(V2::cross(&v0, &v1), -2.0);
        assert_eq!(V2::winding(&v0, &v1, &v0), 0.0);
        assert_eq!(V2::winding(&v0, &v1, &v2), -2.0);
        assert_eq!(V2::winding(&v2, &v1, &v0), 2.0);
    }
}
