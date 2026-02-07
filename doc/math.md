# Math Conventions

## Coordinate System

* Right-handed coordinate system.
* Column-major matrices (OpenGL-style).
* Column vectors are assumed by default.
* Vector transformation uses: `v' = M · v`

## Vectors (V3)

* Cross product follows the right-hand rule.

## Matrices (M3x3)

* Stored in column-major order.
* Represent active linear transforms.
* Rotation matrices are expected to be orthonormal with determinant +1.

## Quaternions (Q)

* Stored as (x, y, z, w) where (x, y, z) is the vector part.
* Represent active rotations.
* Vector rotation is performed as: `v' = q · v · q*`
* q and -q represent the same rotation.
* Quaternions are expected to be normalized when used for rotation.
* from_axis_angle(axis, angle) assumes axis is normalized.
* Conversion from rotation matrices assumes column-major input and internally compensates for row-major formulas.
