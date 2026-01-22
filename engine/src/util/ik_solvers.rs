use crate::v2d::{v2::V2, v3::V3};

// ----------------------------------------------------------------------------
// 2D IK solver - finds the middle joint position given two endpoints and constraint length
pub fn solve_ik_2d(v0: &V2, v1: &V2, constraint_length: f32) -> V2 {
    let k = 0.5 * (*v0 + *v1);
    let n = (*v0 - *v1).perpendicular();
    let l2 = n.length2();

    let a2 = 0.25 * l2;
    let c2 = constraint_length * constraint_length;
    let d2 = c2 - a2;

    if d2 > 0.001 {
        let b = (d2 / l2).sqrt();
        k + b * n
    } else {
        // Constraint cannot be satisfied, extend the chain
        let n0 = *v0 - *v1;
        let n1 = (constraint_length / l2.sqrt()) * n0;
        *v1 + n1
    }
}

// ----------------------------------------------------------------------------
// 3D IK solver - finds the middle joint position given two endpoints, constraint length,
// and a pole vector that indicates which direction the joint should bend
pub fn solve_ik_3d(v0: &V3, v1: &V3, constraint_length: f32, pole: &V3) -> V3 {
    let k = 0.5 * (*v0 + *v1);
    let bone_dir = *v0 - *v1;

    // Project pole vector onto plane perpendicular to bone direction
    let pole_proj = *pole - bone_dir * (pole.dot(&bone_dir) / bone_dir.length2());
    let n = pole_proj.norm();

    let l2 = bone_dir.length2();
    let a2 = 0.25 * l2;
    let c2 = constraint_length * constraint_length;
    let d2 = c2 - a2;

    if d2 > 0.001 {
        let b = (d2 / l2).sqrt();
        k + b * n
    } else {
        // Constraint cannot be satisfied, extend the chain
        let n0 = bone_dir;
        let n1 = (constraint_length / l2.sqrt()) * n0;
        *v1 + n1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ik_solver() {
        let hip = V2::new([0.0, 1.0]);
        let foot = V2::new([0.0, 0.0]);
        let leg_length = 0.6;

        let knee = solve_ik_2d(&hip, &foot, leg_length);

        // Knee should be between hip and foot
        assert!(knee.x0().abs() > 0.0); // Knee bends outward
        assert!(knee.x1() < 1.0 && knee.x1() > 0.0);

        // Check constraint satisfaction
        let upper_dist = V2::distance(&hip, &knee);
        let lower_dist = V2::distance(&knee, &foot);
        assert!((upper_dist - leg_length).abs() < 0.01);
        assert!((lower_dist - leg_length).abs() < 0.01);
    }

    #[test]
    fn test_ik_solver_3d() {
        let hip = V3::new([0.0, 1.0, 0.0]);
        let foot = V3::new([0.0, 0.0, 0.0]);
        let leg_length = 0.6;
        let pole = V3::new([1.0, 0.5, 0.0]); // Bend knee forward

        let knee = solve_ik_3d(&hip, &foot, leg_length, &pole);

        // Knee should be between hip and foot in Y
        assert!(knee.x1() < 1.0 && knee.x1() > 0.0);

        // Knee should bend toward pole direction (positive X)
        assert!(knee.x0() > 0.0);

        // Check constraint satisfaction
        let upper_dist = V3::distance(&hip, &knee);
        let lower_dist = V3::distance(&knee, &foot);
        assert!((upper_dist - leg_length).abs() < 0.01);
        assert!((lower_dist - leg_length).abs() < 0.01);
    }
}
