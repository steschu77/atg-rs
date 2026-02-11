use crate::v2d::float_eq::float_eq_rel;
use crate::v2d::{m3x3::M3x3, v3::V3};

// ----------------------------------------------------------------------------
// Rotates a vector `v` around an arbitrary axis by a specified angle (in radians).
// Uses Rodrigues' rotation formula.
pub fn rotate_axis(v: V3, axis: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    v * c + axis.cross(v) * s + axis * axis.dot(v) * (1.0 - c)
}

// ----------------------------------------------------------------------------
#[rustfmt::skip]
pub fn rotate_x0(rad: f32) -> M3x3
{
    let (s, c) = rad.sin_cos();
    M3x3::new([
        1.0, 0.0, 0.0,
        0.0,   c,  -s,
        0.0,   s,   c,
    ])
}

// ----------------------------------------------------------------------------
#[rustfmt::skip]
pub fn rotate_x1(rad: f32) -> M3x3
{
    let (s, c) = rad.sin_cos();
    M3x3::new([
          c, 0.0,   s,
        0.0, 1.0, 0.0,
         -s, 0.0,   c,
    ])
}

// ----------------------------------------------------------------------------
#[rustfmt::skip]
pub fn rotate_x2(rad: f32) -> M3x3 {
    let (s, c) = rad.sin_cos();
    M3x3::new([
          c,  -s, 0.0,
          s,   c, 0.0,
        0.0, 0.0, 1.0,
    ])
}

// ----------------------------------------------------------------------------
#[rustfmt::skip]
pub fn rotate(v: V3, rad: f32) -> M3x3 {
    let (s, c) = rad.sin_cos();

    let vs = s * v;

    let a0 = v.x0() * v.x0();
    let a1 = v.x1() * v.x1();
    let a2 = v.x2() * v.x2();
    let a3 = v.x0() * v.x1();
    let a4 = v.x1() * v.x2();
    let a5 = v.x2() * v.x0();

    let c0 = a0 - a0 * c;
    let c1 = a1 - a1 * c;
    let c2 = a2 - a2 * c;
    let c3 = a3 - a3 * c;
    let c4 = a4 - a4 * c;
    let c5 = a5 - a5 * c;

    let b00 = c0 + c;
    let b01 = c3 - vs.x2();
    let b02 = c5 + vs.x1();
    let b10 = c3 + vs.x2();
    let b11 = c1 + c;
    let b12 = c4 - vs.x0();
    let b20 = c5 - vs.x1();
    let b21 = c4 + vs.x0();
    let b22 = c2 + c;

    M3x3::new([
        b00, b01, b02,
        b10, b11, b12,
        b20, b21, b22,
    ])
}

// ----------------------------------------------------------------------------
// Constructs an orthonormal, right-handed basis from a single direction vector.
// Guarantees
// - The specified axis is preserved
// Non-guarantees
// - The orientation of the remaining two axes is not fixed.
pub fn basis_from_x0(x0: V3) -> M3x3 {
    debug_assert!(float_eq_rel(x0.length2(), 1.0), "{x0} is not a unit vector");
    let x2 = x0.cross(approx_orthogonal_axis(x0)).norm();
    let x1 = x2.cross(x0).norm();
    M3x3::from_cols(x0, x1, x2)
}

// ----------------------------------------------------------------------------
pub fn basis_from_x1(x1: V3) -> M3x3 {
    debug_assert!(float_eq_rel(x1.length2(), 1.0), "{x1} is not a unit vector");
    let x0 = x1.cross(approx_orthogonal_axis(x1)).norm();
    let x2 = x0.cross(x1).norm();
    M3x3::from_cols(x0, x1, x2)
}

// ----------------------------------------------------------------------------
pub fn basis_from_x2(x2: V3) -> M3x3 {
    debug_assert!(float_eq_rel(x2.length2(), 1.0), "{x2} is not a unit vector");
    let x1 = x2.cross(approx_orthogonal_axis(x2)).norm();
    let x0 = x1.cross(x2).norm();
    M3x3::from_cols(x0, x1, x2)
}

// ----------------------------------------------------------------------------
#[allow(clippy::collapsible_else_if)]
#[allow(clippy::excessive_precision)]
fn approx_orthogonal_axis(v: V3) -> V3 {
    debug_assert!(float_eq_rel(v.length2(), 1.0), "{v} is not a unit vector");
    const INV_SQRT_3: f32 = 0.57735026919; // 1 / √3
    if v.x0().abs() < INV_SQRT_3 {
        V3::X0
    } else if v.x1().abs() < INV_SQRT_3 {
        V3::X1
    } else {
        V3::X2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basis_preserves_axis() {
        let v = V3::uniform(1.0 / 3.0_f32.sqrt());

        let m0 = basis_from_x0(v);
        let m1 = basis_from_x1(v);
        let m2 = basis_from_x2(v);

        let r0 = m0 * V3::X0;
        let r1 = m1 * V3::X1;
        let r2 = m2 * V3::X2;

        assert_eq!(r0, v);
        assert_eq!(r1, v);
        assert_eq!(r2, v);
    }

    #[test]
    fn basis_is_orthonormal() {
        let v = V3::uniform(1.0 / 3.0_f32.sqrt());
        assert!(basis_from_x0(v).is_orthonormal());
        assert!(basis_from_x1(v).is_orthonormal());
        assert!(basis_from_x2(v).is_orthonormal());
    }
}
