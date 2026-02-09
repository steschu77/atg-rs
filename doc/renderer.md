# Renderer Conventions

## Coordinate System

* Right-handed coordinate system.
* Column-major matrices.
* +X points right, +Y points up, +Z points forward.
* OpenGL, Blender and glTF compatible.

## Units & Scale

* All spatial units are expressed in meters.
* Time-based quantities (velocity, acceleration) are expressed in SI units.

## Transforms & Spaces

* Scene nodes use local-space TRS decomposition:
  * Translation: V3
  * Rotation: quaternion (x, y, z, w)
  * Scale: V3

* Local transform composition order is: `M_local = T · R · S`
* World transforms are derived via hierarchical composition.

## Mesh Data

* Vertex positions are defined in object-local space.
* Triangle winding order is counter-clockwise (CCW).
* Front faces are CCW when viewed from the outside.

## Normals, Tangents & Bitangents

* Normals are expected to be unit-length and in object-local space.
* Tangent stored as V4
  * xyz = tangent direction
  * w = bitangent sign
* Bitangent reconstruction is performed as: `B = cross(N, T.xyz) * T.w`
* Tangent space is right-handed.

## Projection & Clip Space

* Normalized device coordinates are assumed:
  * X, Y ∈ [-1, 1]
  * Z ∈ [0, 1]
* Renderer adapts to OpenGL clip space via: projection matrix adjustment
