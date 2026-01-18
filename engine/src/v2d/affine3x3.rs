use crate::v2d::v3::V3;

// ----------------------------------------------------------------------------
pub fn rotate_axis(v: &V3, axis: &V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    *v * c + axis.cross(v) * s + *axis * axis.dot(v) * (1.0 - c)
}
