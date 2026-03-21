## MODIFIED Requirements

### Requirement: CPU-Side Mesh Data
The system SHALL provide a `MeshData` struct containing `positions: Vec<Vec3>`, `normals: Vec<Vec3>`, `texcoords: Vec<Vec2>`, `tangents: Vec<Vec4>`, and `indices: Vec<u32>`.

`MeshData` SHALL provide `vertex_count()` and `index_count()` accessors.

`MeshData` SHALL validate on construction that `positions.len() == normals.len() == texcoords.len()`, returning an error if lengths are inconsistent.

#### Scenario: Mesh data integrity
- **WHEN** `MeshData` is constructed from a glTF file
- **THEN** `positions.len() == normals.len() == texcoords.len()` and all indices are within `0..vertex_count()`

#### Scenario: Inconsistent array lengths
- **WHEN** `MeshData` is constructed with `positions.len() != normals.len()`
- **THEN** the construction returns an error describing the length mismatch

## ADDED Requirements

### Requirement: Skeletal Animation TRS Composition
The system SHALL compute bone matrices by accumulating Translation, Rotation, and Scale channels independently per joint, then composing them in standard TRS order: `T × R × S`.

When a joint has both Translation and Rotation channels animated, the system SHALL NOT left-multiply each channel's matrix onto the local transform sequentially. Instead, it SHALL collect the final T, R, S values and compose a single `Mat4::from_scale_rotation_translation(s, r, t)`.

#### Scenario: Joint with translation and rotation
- **WHEN** a joint is animated with Translation(2, 0, 0) and Rotation(45° around Y)
- **THEN** the resulting local transform is equivalent to `Mat4::from_translation(2,0,0) * Mat4::from_rotation_y(45°)`, i.e., translate first then rotate in world space

#### Scenario: Joint with all three channels
- **WHEN** a joint has Translation, Rotation, and Scale channels
- **THEN** the composed matrix equals `T × R × S` matching the glTF specification

### Requirement: Cubic Spline Interpolation
The system SHALL implement glTF cubic spline interpolation for animation keyframes, using the cubic Hermite spline formula with in-tangent and out-tangent data.

The interpolation formula SHALL be: `p(t) = (2t³ - 3t² + 1)v₀ + (t³ - 2t² + t)b₀ + (-2t³ + 3t²)v₁ + (t³ - t²)a₁`

where `v₀`, `v₁` are keyframe values, `b₀` is the out-tangent of keyframe 0 scaled by delta time, and `a₁` is the in-tangent of keyframe 1 scaled by delta time.

#### Scenario: Cubic spline vs linear
- **WHEN** an animation channel uses `CubicSpline` interpolation
- **THEN** the sampled value follows a smooth cubic curve between keyframes, not a linear segment

#### Scenario: Tangent influence
- **WHEN** an animation has keyframes with non-zero tangent values
- **THEN** the interpolated curve overshoots or undershoots keyframe values according to the tangent directions

### Requirement: Texture Format Handling
The system SHALL support converting the following glTF texture formats to RGBA8: R8G8B8A8, R8G8B8, R8G8, R8, R16G16B16A16.

For unsupported formats, the system SHALL return a descriptive error instead of silently returning `None`.

#### Scenario: R8G8B8 to RGBA8 conversion
- **WHEN** a glTF texture uses R8G8B8 format (3 channels, no alpha)
- **THEN** the system converts it to RGBA8 with alpha set to 255

#### Scenario: R16G16B16A16 conversion
- **WHEN** a glTF texture uses 16-bit per channel format
- **THEN** the system downsamples to 8-bit RGBA by dividing values by 257 (65535/255)

#### Scenario: Unsupported format error
- **WHEN** a glTF texture uses an unrecognized format
- **THEN** the system returns an error with the format name, not a silent `None`
