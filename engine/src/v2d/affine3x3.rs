use crate::v2d::{m3x3::M3x3, v3::V3};

// ----------------------------------------------------------------------------
pub fn rotate_axis(v: &V3, axis: &V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    *v * c + axis.cross(v) * s + *axis * axis.dot(v) * (1.0 - c)
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
pub fn rotate(v: &V3, rad: f32) -> M3x3 {
    let (s, c) = rad.sin_cos();

    let vs = s * *v;

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
