## ADDED Requirements

### Requirement: Spatial Primitives
The system SHALL provide `Aabb` (axis-aligned bounding box) in `anvilkit-core::math` with `min`/`max` fields, `contains_point()`, `intersects()`, and `from_center_half_extents()` methods.

The system SHALL provide raycast utility functions (`screen_to_ray`, `ray_plane_intersection`, `ray_sphere_intersection`) in `anvilkit-core::math`, as they are pure math with no GPU dependency.

#### Scenario: AABB intersection
- **WHEN** two AABBs overlap in all three axes
- **THEN** `aabb_a.intersects(&aabb_b)` returns true

#### Scenario: Screen-to-ray unprojection
- **WHEN** `screen_to_ray(screen_pos, window_size, view_proj_inverse)` is called
- **THEN** a ray origin and direction in world space are returned
