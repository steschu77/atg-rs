// Quaternion
use super::float_eq::float_eq_rel;
use super::{m3x3::M3x3, m4x4::M4x4, v3::V3};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// ----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub struct Q {
    m: [f32; 4],
}

// ----------------------------------------------------------------------------
impl Default for Q {
    fn default() -> Self {
        Q::identity()
    }
}

// ----------------------------------------------------------------------------
impl PartialEq for Q {
    fn eq(&self, rhs: &Self) -> bool {
        float_eq_rel(self.x0(), rhs.x0())
            && float_eq_rel(self.x1(), rhs.x1())
            && float_eq_rel(self.x2(), rhs.x2())
            && float_eq_rel(self.x3(), rhs.x3())
    }
}

// ----------------------------------------------------------------------------
// Q + Q -> Q
impl Add for Q {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Q::new([
            self.x0() + rhs.x0(),
            self.x1() + rhs.x1(),
            self.x2() + rhs.x2(),
            self.x3() + rhs.x3(),
        ])
    }
}

// ----------------------------------------------------------------------------
// Q - Q -> Q
impl Sub for Q {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Q::new([
            self.x0() - rhs.x0(),
            self.x1() - rhs.x1(),
            self.x2() - rhs.x2(),
            self.x3() - rhs.x3(),
        ])
    }
}

// ----------------------------------------------------------------------------
// Q * f32 -> Q
impl Mul<f32> for Q {
    type Output = Self;

    fn mul(self, s: f32) -> Self {
        Q::new([self.x0() * s, self.x1() * s, self.x2() * s, self.x3() * s])
    }
}

// ----------------------------------------------------------------------------
// f32 * Q -> Q
impl Mul<Q> for f32 {
    type Output = Q;

    fn mul(self, q: Q) -> Q {
        q * self
    }
}

// ----------------------------------------------------------------------------
// f32 * Q (ref)
impl Mul<&Q> for f32 {
    type Output = Q;

    fn mul(self, q: &Q) -> Q {
        *q * self
    }
}

// ----------------------------------------------------------------------------
// Q * Q -> Q (Hamilton product)
impl Mul for Q {
    type Output = Q;

    #[rustfmt::skip]
    fn mul(self, rhs: Q) -> Q {
        Q::new([
            self.x3() * rhs.x0() + self.x0() * rhs.x3() + self.x1() * rhs.x2() - self.x2() * rhs.x1(),
            self.x3() * rhs.x1() + self.x1() * rhs.x3() + self.x2() * rhs.x0() - self.x0() * rhs.x2(),
            self.x3() * rhs.x2() + self.x2() * rhs.x3() + self.x0() * rhs.x1() - self.x1() * rhs.x0(),
            self.x3() * rhs.x3() - self.x0() * rhs.x0() - self.x1() * rhs.x1() - self.x2() * rhs.x2(),
        ])
    }
}

// ----------------------------------------------------------------------------
impl Neg for Q {
    type Output = Self;

    fn neg(self) -> Self {
        Q::new([-self.x0(), -self.x1(), -self.x2(), -self.x3()])
    }
}

// ----------------------------------------------------------------------------
impl AddAssign for Q {
    fn add_assign(&mut self, rhs: Self) {
        self.m[0] += rhs.x0();
        self.m[1] += rhs.x1();
        self.m[2] += rhs.x2();
        self.m[3] += rhs.x3();
    }
}

// ----------------------------------------------------------------------------
impl SubAssign for Q {
    fn sub_assign(&mut self, rhs: Self) {
        self.m[0] -= rhs.x0();
        self.m[1] -= rhs.x1();
        self.m[2] -= rhs.x2();
        self.m[3] -= rhs.x3();
    }
}

// ----------------------------------------------------------------------------
impl MulAssign<f32> for Q {
    fn mul_assign(&mut self, s: f32) {
        self.m[0] *= s;
        self.m[1] *= s;
        self.m[2] *= s;
        self.m[3] *= s;
    }
}

// ----------------------------------------------------------------------------
impl Q {
    pub const fn new(m: [f32; 4]) -> Self {
        Q { m }
    }

    pub const fn identity() -> Self {
        Q::new([0.0, 0.0, 0.0, 1.0])
    }

    // ------------------------------------------------------------------------
    pub const fn x0(&self) -> f32 {
        self.m[0]
    }
    pub const fn x1(&self) -> f32 {
        self.m[1]
    }
    pub const fn x2(&self) -> f32 {
        self.m[2]
    }
    pub const fn x3(&self) -> f32 {
        self.m[3]
    }

    // ------------------------------------------------------------------------
    pub const fn dot(a: &Self, b: &Self) -> f32 {
        a.x0() * b.x0() + a.x1() * b.x1() + a.x2() * b.x2() + a.x3() * b.x3()
    }

    // ------------------------------------------------------------------------
    pub const fn length2(&self) -> f32 {
        Self::dot(self, self)
    }

    // ------------------------------------------------------------------------
    pub fn length(&self) -> f32 {
        self.length2().sqrt()
    }

    // ------------------------------------------------------------------------
    pub fn norm(&self) -> Self {
        let l2 = self.length2();
        if l2 < f32::EPSILON {
            Q::identity()
        } else {
            let inv = 1.0 / l2.sqrt();
            *self * inv
        }
    }

    // ------------------------------------------------------------------------
    pub const fn conjugate(&self) -> Self {
        Q::new([-self.x0(), -self.x1(), -self.x2(), self.x3()])
    }

    // ------------------------------------------------------------------------
    pub fn inverse(&self) -> Self {
        let l2 = self.length2();
        if l2 < f32::EPSILON {
            Q::identity()
        } else {
            self.conjugate() * (1.0 / l2)
        }
    }

    // ----------------------------------------------------------------------------
    // NLERP: normalized linear interpolation
    pub fn nlerp(q0: Self, q1: Self, t: f32) -> Self {
        let dot = Q::dot(&q0, &q1);
        let q1 = if dot < 0.0 { -q1 } else { q1 };
        (q0 * (1.0 - t) + q1 * t).norm()
    }

    // ----------------------------------------------------------------------------
    // SLERP: spherical linear interpolation
    pub fn slerp(a: Self, b: Self, t: f32) -> Self {
        let mut b = b;
        let mut c = Q::dot(&a, &b);

        // Take shortest path
        if c < 0.0 {
            b = -b;
            c = -c;
        }

        // If nearly parallel, fall back to nlerp
        if c > 0.9995 {
            return Q::nlerp(a, b, t);
        }

        let th = c.acos();
        let s = th.sin();

        let w0 = ((1.0 - t) * th).sin() / s;
        let w1 = (t * th).sin() / s;

        a * w0 + b * w1
    }

    // ----------------------------------------------------------------------------
    // Convert to a 3×3 rotation matrix (column-major)
    #[rustfmt::skip]
    pub fn as_mat3x3(&self) -> M3x3 {
        let x2 = self.x0() + self.x0();
        let y2 = self.x1() + self.x1();
        let z2 = self.x2() + self.x2();

        let xx = self.x0() * x2;
        let yy = self.x1() * y2;
        let zz = self.x2() * z2;

        let xy = self.x0() * y2;
        let xz = self.x0() * z2;
        let yz = self.x1() * z2;

        let wx = self.x3() * x2;
        let wy = self.x3() * y2;
        let wz = self.x3() * z2;

        M3x3::new([
            1.0 - (yy + zz), xy + wz, xz - wy,
            xy - wz, 1.0 - (xx + zz), yz + wx,
            xz + wy, yz - wx, 1.0 - (xx + yy),
        ])
    }

    // ----------------------------------------------------------------------------
    // Convert to a 4×4 rotation matrix (column-major)
    #[rustfmt::skip]
    pub fn as_mat4x4(&self) -> M4x4 {
        let x2 = self.x0() + self.x0();
        let y2 = self.x1() + self.x1();
        let z2 = self.x2() + self.x2();

        let xx = self.x0() * x2;
        let yy = self.x1() * y2;
        let zz = self.x2() * z2;

        let xy = self.x0() * y2;
        let xz = self.x0() * z2;
        let yz = self.x1() * z2;

        let wx = self.x3() * x2;
        let wy = self.x3() * y2;
        let wz = self.x3() * z2;

        M4x4::new([
            1.0 - (yy + zz), xy + wz, xz - wy, 0.0,
            xy - wz, 1.0 - (xx + zz), yz + wx, 0.0,
            xz + wy, yz - wx, 1.0 - (xx + yy), 0.0,
                  0.0,        0.0,        0.0, 1.0,
        ])
    }

    // ------------------------------------------------------------------------
    // Rotate a vector
    pub fn rotate(&self, v: &V3) -> V3 {
        let qv = Q::new([v.x0(), v.x1(), v.x2(), 0.0]);
        let r = *self * qv * self.inverse();
        V3::new([r.x0(), r.x1(), r.x2()])
    }

    // ------------------------------------------------------------------------
    pub fn from_axis_angle(axis: &V3, angle: f32) -> Self {
        let half = angle * 0.5;
        let (s, c) = half.sin_cos();
        Q::new([axis.x0() * s, axis.x1() * s, axis.x2() * s, c])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_float_eq;
    use std::f32::consts::PI;

    #[test]
    fn test_axis_angle() {
        let axis = V3::new([1.0, 1.0, 1.0]).norm();
        let q = Q::from_axis_angle(&axis, PI);
        assert_float_eq!(q.length(), 1.0);
    }

    #[test]
    fn test_rotate_vector_axis_angle() {
        let axis = V3::new([0.0, 1.0, 0.0]);
        let v = V3::new([1.0, 1.0, 1.0]);
        let q = Q::from_axis_angle(&axis, PI);
        let r = q.rotate(&v);
        assert_eq!(r, V3::new([-1.0, 1.0, -1.0]));
    }

    /// Rotation on axis parallel to vector direction should have no effect
    #[test]
    fn test_rotate_vector_axis_angle_same_axis() {
        let v = V3::new([1.0, 1.0, 1.0]);
        let axis = V3::new([1.0, 1.0, 1.0]).norm();
        let q = Q::from_axis_angle(&axis, 31.41);
        let r = q.rotate(&v);
        assert_eq!(r, v);
    }

    // #[test]
    // fn test_rotation_from_to() {
    //     let a = V3::new([1.0, 1.0, 1.0]);
    //     let b = V3::new([-1.0, -1.0, -1.0]);
    //     let q = super::rotation_from_to(a, b);
    //     let a_prime = q.rotate(a);
    //     assert_eq!(a_prime, a);
    // }
}
