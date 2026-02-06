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
        let dot = self.x0() * rhs.x0()
            + self.x1() * rhs.x1()
            + self.x2() * rhs.x2()
            + self.x3() * rhs.x3();
        float_eq_rel(dot.abs(), 1.0)
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
    pub fn nlerp(&self, q1: &Self, t: f32) -> Self {
        let dot = Q::dot(self, q1);
        let q1 = if dot < 0.0 { -*q1 } else { *q1 };
        (*self * (1.0 - t) + q1 * t).norm()
    }

    // ----------------------------------------------------------------------------
    // SLERP: spherical linear interpolation
    pub fn slerp(&self, b: &Self, t: f32) -> Self {
        let mut b = *b;
        let mut c = Q::dot(self, &b);

        // Take shortest path
        if c < 0.0 {
            b = -b;
            c = -c;
        }

        // If nearly parallel, fall back to nlerp
        if c > 0.9995 {
            return self.nlerp(&b, t);
        }

        let th = c.acos();
        let s = th.sin();

        let w0 = ((1.0 - t) * th).sin() / s;
        let w1 = (t * th).sin() / s;

        *self * w0 + b * w1
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
            1.0 - (yy + zz), xy - wz, xz + wy,
            xy + wz, 1.0 - (xx + zz), yz - wx,
            xz - wy, yz + wx, 1.0 - (xx + yy),
        ])
    }

    // ----------------------------------------------------------------------------
    // Convert to a 4×4 rotation matrix (column-major)
    #[rustfmt::skip]
    pub fn as_mat4x4(&self) -> M4x4 {
        let m3x3 = self.as_mat3x3();

        M4x4::new([
            m3x3.x00(), m3x3.x10(), m3x3.x20(), 0.0,
            m3x3.x01(), m3x3.x11(), m3x3.x21(), 0.0,
            m3x3.x02(), m3x3.x12(), m3x3.x22(), 0.0,
            0.0,        0.0,        0.0,        1.0,
        ])
    }

    // ------------------------------------------------------------------------
    // Rotate a vector
    pub fn rotate(&self, v: &V3) -> V3 {
        let qv = Q::new([v.x0(), v.x1(), v.x2(), 0.0]);
        let r = self.conjugate() * qv * *self;
        V3::new([r.x0(), r.x1(), r.x2()])
    }

    // ------------------------------------------------------------------------
    // Rotates a vector by the inverse of this quaternion.
    pub fn inv_rotate(&self, v: &V3) -> V3 {
        let qv = Q::new([v.x0(), v.x1(), v.x2(), 0.0]);
        let r = *self * qv * self.conjugate();
        V3::new([r.x0(), r.x1(), r.x2()])
    }

    // ------------------------------------------------------------------------
    pub fn from_axis_angle(axis: &V3, angle: f32) -> Self {
        let half = angle * 0.5;
        let (s, c) = half.sin_cos();
        Q::new([axis.x0() * s, axis.x1() * s, axis.x2() * s, c])
    }

    // ------------------------------------------------------------------------
    pub fn from_mat3(m: &M3x3) -> Self {
        let trace = m.x00() + m.x11() + m.x22();

        let q = if trace > 0.0 {
            let t = trace + 1.0;
            let s = 0.5 / t.sqrt();
            Q::new([
                (m.x12() - m.x21()) * s,
                (m.x20() - m.x02()) * s,
                (m.x01() - m.x10()) * s,
                0.25 / s,
            ])
        } else if m.x00() > m.x11() && m.x00() > m.x22() {
            let t = 1.0 + m.x00() - m.x11() - m.x22();
            let s = 0.5 / t.sqrt();
            Q::new([
                0.25 / s,
                (m.x10() + m.x01()) * s,
                (m.x20() + m.x02()) * s,
                (m.x12() - m.x21()) * s,
            ])
        } else if m.x11() > m.x22() {
            let t = 1.0 - m.x00() + m.x11() - m.x22();
            let s = 0.5 / t.sqrt();
            Q::new([
                (m.x10() + m.x01()) * s,
                0.25 / s,
                (m.x21() + m.x12()) * s,
                (m.x20() - m.x02()) * s,
            ])
        } else {
            let t = 1.0 - m.x00() - m.x11() + m.x22();
            let s = 0.5 / t.sqrt();
            Q::new([
                (m.x20() + m.x02()) * s,
                (m.x21() + m.x12()) * s,
                0.25 / s,
                (m.x01() - m.x10()) * s,
            ])
        };

        q.norm()
    }

    // ------------------------------------------------------------------------
    pub fn from_axes(x_axis: &V3, y_axis: &V3, z_axis: &V3) -> Self {
        let m = M3x3::from_cols(*x_axis, *y_axis, *z_axis);
        debug_assert!(m.det() > 0.0, "Basis must be right-handed");

        let q = Q::from_mat3(&m);
        debug_assert_eq!(q.rotate(&V3::X0), *x_axis);
        debug_assert_eq!(q.rotate(&V3::X1), *y_axis);
        debug_assert_eq!(q.rotate(&V3::X2), *z_axis);

        q
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

    #[test]
    fn test_rotate_vector_axis_angle_inv() {
        let axis = V3::new([0.0, 1.0, 0.0]);
        let v = V3::new([1.0, 1.0, 1.0]);
        let q = Q::from_axis_angle(&axis, PI);
        let u = q.inv_rotate(&v);
        let r = q.rotate(&u);
        assert_eq!(r, v);
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

    #[test]
    fn mat3_to_quat_identity() {
        let m = M3x3::identity();
        let q = Q::from_mat3(&m);
        let expected = Q::new([0.0, 0.0, 0.0, 1.0]);
        assert_eq!(q, expected);
    }

    #[test]
    #[rustfmt::skip]
    fn mat3_to_quat_rot_x_90() {
        let s = 0.5_f32.sqrt();
        let m = M3x3::new([
            1.0, 0.0,  0.0,
            0.0, 0.0, -1.0,
            0.0, 1.0,  0.0
        ]);
        let q = Q::from_mat3(&m);
        let expected = Q::new([s, 0.0, 0.0, s]);
        assert_eq!(q, expected);
    }

    #[test]
    #[rustfmt::skip]
    fn mat3_to_quat_rot_y_90() {
        let s = 0.5_f32.sqrt();
        let m = M3x3::new([
             0.0, 0.0, 1.0,
             0.0, 1.0, 0.0,
            -1.0, 0.0, 0.0
        ]);
        let q = Q::from_mat3(&m);
        let expected = Q::new([0.0, s, 0.0, s]);
        assert_eq!(q, expected);
    }

    #[test]
    #[rustfmt::skip]
    fn mat3_to_quat_rot_z_90() {
        let s = 0.5_f32.sqrt();
        let m = M3x3::new([
            0.0, -1.0, 0.0,
            1.0,  0.0, 0.0,
            0.0,  0.0, 1.0
        ]);
        let q = Q::from_mat3(&m);
        let expected = Q::new([0.0, 0.0, s, s]);
        assert_eq!(q, expected);
    }

    #[test]
    #[rustfmt::skip]
    fn mat3_to_quat_rot_180_x() {
        let m = M3x3::new([
            1.0,  0.0,  0.0,
            0.0, -1.0,  0.0,
            0.0,  0.0, -1.0
        ]);
        let q = Q::from_mat3(&m);
        let expected = Q::new([1.0, 0.0, 0.0, 0.0]);
        assert_eq!(q, expected);
    }

    #[test]
    fn mat3_quat_roundtrip() {
        let q = Q::new([0.3, 0.4, 0.0, 0.8]).norm();
        let r = Q::from_mat3(&q.as_mat3x3());

        let v = V3::new([1.0, 2.0, 3.0]);
        let v_rot_q = q.rotate(&v);
        let v_rot_r = r.rotate(&v);
        assert_eq!(v_rot_q, v_rot_r);
    }

    #[test]
    fn mat3_quat_rotate() {
        let q = Q::new([0.3, 0.4, 0.0, 0.8]).norm();
        let m = q.as_mat3x3();

        let v = V3::new([1.0, 2.0, 3.0]);
        let v_rot_q = q.rotate(&v);
        let v_rot_m = m * v;
        assert_eq!(v_rot_q, v_rot_m);
    }

    #[test]
    fn axis_quat_rotate() {
        let x_axis = V3::new([0.6, 0.8, 0.0]);
        let y_axis = V3::new([-0.8, 0.6, 0.0]);
        let z_axis = V3::new([0.0, 0.0, 1.0]);
        let q = Q::from_axes(&x_axis, &y_axis, &z_axis);

        let v_rot_q = q.rotate(&[1.0, 0.0, 0.0].into());
        assert_eq!(v_rot_q, x_axis);

        let v_rot_q = q.rotate(&[0.0, 1.0, 0.0].into());
        assert_eq!(v_rot_q, y_axis);

        let v_rot_q = q.rotate(&[0.0, 0.0, 1.0].into());
        assert_eq!(v_rot_q, z_axis);
    }

    #[test]
    fn axis_quat_rotate_2() {
        let x_axis = V3::new([-0.6544649, -0.3786178, -0.6544649]);
        let y_axis = V3::new([-0.17025319, 0.9171547, -0.3603346]);
        let z_axis = -V3::new([-0.73667467, 0.12440162, 0.66470647]);
        let q = Q::from_axes(&x_axis, &y_axis, &z_axis);

        let v_rot_q = q.rotate(&[1.0, 0.0, 0.0].into());
        assert_eq!(v_rot_q, x_axis);

        let v_rot_q = q.rotate(&[0.0, 1.0, 0.0].into());
        assert_eq!(v_rot_q, y_axis);

        let v_rot_q = q.rotate(&[0.0, 0.0, 1.0].into());
        assert_eq!(v_rot_q, z_axis);
    }
}
