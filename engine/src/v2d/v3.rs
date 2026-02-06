use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use super::Positive;
use super::float_eq::float_eq_rel;
use super::v2::V2;
use super::v4::V4;

// ----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct V3 {
    m: [f32; 3],
}

// ----------------------------------------------------------------------------
impl Default for V3 {
    fn default() -> Self {
        V3::zero()
    }
}

// ----------------------------------------------------------------------------
impl fmt::Display for V3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "V3({:.2}, {:.2}, {:.2})",
            self.x0(),
            self.x1(),
            self.x2()
        )
    }
}

// ----------------------------------------------------------------------------
impl PartialEq for V3 {
    #[rustfmt::skip]
    fn eq(&self, rhs: &Self) -> bool {
        float_eq_rel(self.x0(), rhs.x0()) &&
        float_eq_rel(self.x1(), rhs.x1()) &&
        float_eq_rel(self.x2(), rhs.x2())
    }
}

// ----------------------------------------------------------------------------
impl Positive for V3 {
    fn is_positive(&self) -> bool {
        self.x0().is_positive() && self.x1().is_positive() && self.x2().is_positive()
    }
}

// ----------------------------------------------------------------------------
impl Add for V3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x0 = self.x0() + rhs.x0();
        let x1 = self.x1() + rhs.x1();
        let x2 = self.x2() + rhs.x2();
        V3::new([x0, x1, x2])
    }
}

// ----------------------------------------------------------------------------
impl Sub for V3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x0 = self.x0() - rhs.x0();
        let x1 = self.x1() - rhs.x1();
        let x2 = self.x2() - rhs.x2();
        V3::new([x0, x1, x2])
    }
}

// ----------------------------------------------------------------------------
// V3 * f32 -> V3
impl Mul<f32> for V3 {
    type Output = Self;

    fn mul(self, s: f32) -> Self {
        let x0 = self.x0() * s;
        let x1 = self.x1() * s;
        let x2 = self.x2() * s;
        V3::new([x0, x1, x2])
    }
}

// ----------------------------------------------------------------------------
// V3 / f32 -> V3
impl Div<f32> for V3 {
    type Output = Self;

    fn div(self, s: f32) -> Self {
        let inv_s = 1.0 / s;
        let x0 = self.x0() * inv_s;
        let x1 = self.x1() * inv_s;
        let x2 = self.x2() * inv_s;
        V3::new([x0, x1, x2])
    }
}

// ----------------------------------------------------------------------------
// f32 * V3 -> V3
impl Mul<V3> for f32 {
    type Output = V3;

    fn mul(self, v: V3) -> V3 {
        let x0 = self * v.x0();
        let x1 = self * v.x1();
        let x2 = self * v.x2();
        V3::new([x0, x1, x2])
    }
}

// ----------------------------------------------------------------------------
// f32 / V3 -> V3
impl Div<V3> for f32 {
    type Output = V3;

    fn div(self, v: V3) -> V3 {
        let x0 = self / v.x0();
        let x1 = self / v.x1();
        let x2 = self / v.x2();
        V3::new([x0, x1, x2])
    }
}

// ----------------------------------------------------------------------------
// V3 * V3 -> f32
impl Mul for V3 {
    type Output = f32;

    fn mul(self, rhs: Self) -> f32 {
        self.x0() * rhs.x0() + self.x1() * rhs.x1() + self.x2() * rhs.x2()
    }
}

// ----------------------------------------------------------------------------
impl Neg for V3 {
    type Output = Self;

    fn neg(self) -> Self {
        V3::new([-self.x0(), -self.x1(), -self.x2()])
    }
}

// ----------------------------------------------------------------------------
impl AddAssign for V3 {
    fn add_assign(&mut self, rhs: Self) {
        self.m[0] += rhs.x0();
        self.m[1] += rhs.x1();
        self.m[2] += rhs.x2();
    }
}

// ----------------------------------------------------------------------------
impl SubAssign for V3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.m[0] -= rhs.x0();
        self.m[1] -= rhs.x1();
        self.m[2] -= rhs.x2();
    }
}

// ----------------------------------------------------------------------------
impl MulAssign<f32> for V3 {
    fn mul_assign(&mut self, s: f32) {
        self.m[0] *= s;
        self.m[1] *= s;
        self.m[2] *= s;
    }
}

// ----------------------------------------------------------------------------
impl From<[f32; 3]> for V3 {
    fn from(m: [f32; 3]) -> Self {
        V3 { m }
    }
}

// ----------------------------------------------------------------------------
impl From<V4> for V3 {
    fn from(v: V4) -> Self {
        V3::new([v.x0(), v.x1(), v.x2()])
    }
}

// ----------------------------------------------------------------------------
impl V3 {
    // ------------------------------------------------------------------------
    pub const fn new(m: [f32; 3]) -> Self {
        V3 { m }
    }

    // ------------------------------------------------------------------------
    pub const fn zero() -> Self {
        V3::new([0.0, 0.0, 0.0])
    }

    // ------------------------------------------------------------------------
    pub const fn one() -> Self {
        V3::new([1.0, 1.0, 1.0])
    }

    // ------------------------------------------------------------------------
    pub const fn uniform(value: f32) -> Self {
        V3::new([value, value, value])
    }

    // ------------------------------------------------------------------------
    pub const fn from_v2(v: &V2, z: f32) -> Self {
        V3::new([v.x0(), v.x1(), z])
    }

    // ------------------------------------------------------------------------
    pub const fn from_slice(m: &[f32; 3]) -> Self {
        V3 { m: *m }
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
    pub const fn with_x2(mut self, value: f32) -> Self {
        self.m[2] = value;
        self
    }

    // ------------------------------------------------------------------------
    pub const X0: V3 = V3::new([1.0, 0.0, 0.0]);
    pub const X1: V3 = V3::new([0.0, 1.0, 0.0]);
    pub const X2: V3 = V3::new([0.0, 0.0, 1.0]);
    pub const ZERO: V3 = V3::zero();
    pub const ONE: V3 = V3::one();

    // ------------------------------------------------------------------------
    pub const fn x0(&self) -> f32 {
        self.m[0]
    }

    // ------------------------------------------------------------------------
    pub const fn x1(&self) -> f32 {
        self.m[1]
    }

    // ------------------------------------------------------------------------
    pub const fn x2(&self) -> f32 {
        self.m[2]
    }

    // ------------------------------------------------------------------------
    pub fn as_array(&self) -> [f32; 3] {
        self.m
    }

    // ------------------------------------------------------------------------
    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }

    // ------------------------------------------------------------------------
    pub const fn length2(&self) -> f32 {
        self.x0() * self.x0() + self.x1() * self.x1() + self.x2() * self.x2()
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
            V3::default()
        } else {
            let inv_l = 1.0 / l2.sqrt();
            let x0 = self.x0() * inv_l;
            let x1 = self.x1() * inv_l;
            let x2 = self.x2() * inv_l;
            V3::new([x0, x1, x2])
        }
    }

    // ------------------------------------------------------------------------
    pub fn abs(&self) -> Self {
        V3::new([self.x0().abs(), self.x1().abs(), self.x2().abs()])
    }

    // ------------------------------------------------------------------------
    pub const fn dot(&self, v1: &Self) -> f32 {
        self.x0() * v1.x0() + self.x1() * v1.x1() + self.x2() * v1.x2()
    }

    // ------------------------------------------------------------------------
    pub const fn cross(&self, v1: &Self) -> Self {
        let x0 = self.x1() * v1.x2() - self.x2() * v1.x1();
        let x1 = self.x2() * v1.x0() - self.x0() * v1.x2();
        let x2 = self.x0() * v1.x1() - self.x1() * v1.x0();
        V3::new([x0, x1, x2])
    }

    // ------------------------------------------------------------------------
    pub fn lerp(&self, other: &V3, t: f32) -> V3 {
        *self + (*other - *self) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v3() {
        let v0 = V3::new([3.0, 4.0, 0.0]);
        let v1 = V3::new([1.0, 2.0, 1.0]);

        assert_eq!(v0.x0(), 3.0);
        assert_eq!(v0.x1(), 4.0);
        assert_eq!(v0.x2(), 0.0);
        assert_eq!(v0 + v1, V3::new([4.0, 6.0, 1.0]));
        assert_eq!(v0 - v1, V3::new([2.0, 2.0, -1.0]));
        assert_eq!(v0 * 2.0, V3::new([6.0, 8.0, 0.0]));
        assert_eq!(v1 / 2.0, V3::new([0.5, 1.0, 0.5]));
        assert_eq!(2.0 * v0, V3::new([6.0, 8.0, 0.0]));
        assert_eq!(2.0 / v1, V3::new([2.0, 1.0, 2.0]));
        assert_eq!(v0 * v1, 11.0);
        assert_eq!(-v0, V3::new([-3.0, -4.0, 0.0]));
        assert_eq!(v0.length2(), 25.0);
        assert_eq!(v0.length(), 5.0);
        assert_eq!(v0.norm(), V3::new([0.6, 0.8, 0.0]));
        assert_eq!(v0.abs(), V3::new([3.0, 4.0, 0.0]));
        assert_eq!(V3::distance(&v0, &v1), 3.0);
        assert_eq!(v0.dot(&v1), 11.0);
        assert_eq!(v0.cross(&v1), V3::new([4.0, -3.0, 2.0]));
        assert_eq!(v0.lerp(&v1, 0.5), V3::new([2.0, 3.0, 0.5]));
        assert!(!v0.is_positive());
        assert!(v1.is_positive());
    }
}
