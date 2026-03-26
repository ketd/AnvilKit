## MODIFIED Requirements

### Requirement: ECS Render Plugin

The system SHALL provide `RenderPlugin` implementing the ECS `Plugin` trait to register rendering systems and resources.

The system SHALL provide rendering components:
- `RenderComponent` — marks an entity for rendering (visible flag, layer)
- `CameraComponent` — camera parameters (FOV, near/far planes, active flag, **projection type, render target**)
- `MeshComponent` — mesh geometry reference (mesh_id, vertex/index counts)
- `MaterialComponent` — material properties (base color, metallic, roughness)

The `CameraComponent` SHALL support both `Projection::Perspective` and `Projection::Orthographic` variants.

The system SHALL support multiple active cameras, each rendering to its own `RenderTarget` (window swapchain or custom texture).

The system SHALL provide `RenderConfig` as a global resource with configurable MSAA sample count, clear color, and default cull mode.

#### Scenario: Plugin registration
- **WHEN** `app.add_plugins(RenderPlugin)` is called
- **THEN** rendering systems and resources are registered in the ECS world

#### Scenario: Orthographic camera
- **WHEN** a `CameraComponent` is created with `Projection::Orthographic { left, right, bottom, top, near, far }`
- **THEN** the view-projection matrix uses orthographic projection

#### Scenario: Multi-camera rendering
- **WHEN** two cameras are active with different render targets
- **THEN** the scene is rendered once per camera to their respective targets

#### Scenario: Configurable MSAA
- **WHEN** `RenderConfig.msaa_samples` is set to 1
- **THEN** MSAA is disabled and no multisample resolve is performed

#### Scenario: Configurable clear color
- **WHEN** `RenderConfig.clear_color` is set to `[0.0, 0.0, 0.0, 1.0]`
- **THEN** the scene pass clears to black instead of the default sky blue

## ADDED Requirements

### Requirement: Post-Processing Pipeline Integration

The system SHALL integrate all implemented post-processing effects (SSAO, DOF, Motion Blur, Color Grading) into the main `render_ecs()` rendering loop, controlled by the `PostProcessSettings` resource.

The post-processing chain SHALL execute in fixed order: SSAO → DOF → Motion Blur → Bloom → Color Grading → Tonemap.

Each effect SHALL be independently enable/disable via its corresponding `Option<Settings>` field.

#### Scenario: SSAO integration
- **WHEN** `PostProcessSettings.ssao` is `Some(SsaoSettings { quality: High, .. })`
- **THEN** the SSAO pass executes after the scene pass and the AO factor is applied during tonemapping

#### Scenario: Full pipeline
- **WHEN** all five post-processing effects are enabled
- **THEN** they execute in order (SSAO → DOF → Motion Blur → Bloom → Color Grading) before the final tonemap pass

### Requirement: Mipmap Generation

The system SHALL automatically generate mipmaps for textures created via `create_texture()` and `create_texture_linear()`.

Mipmap generation SHALL use a blit-chain approach (downscaling each mip level from the previous one using linear filtering).

The sampler SHALL use `FilterMode::Linear` for `mipmap_filter` when mipmaps are available.

#### Scenario: Texture with mipmaps
- **WHEN** a 1024x1024 texture is created
- **THEN** a full mip chain (1024 → 512 → 256 → ... → 1) is generated and the texture has `mip_level_count = floor(log2(max(w,h))) + 1`

#### Scenario: Oblique surface rendering
- **WHEN** a textured surface is viewed at a steep angle
- **THEN** the appropriate mip level is sampled, eliminating aliasing artifacts

### Requirement: CSM Camera FOV Correctness

The cascade shadow map system SHALL use the actual `CameraComponent.fov` value when computing cascade frustum splits, not a hardcoded value.

The CSM system SHALL use the same coordinate system handedness (left-handed) as the main camera projection.

#### Scenario: Wide FOV camera
- **WHEN** the camera FOV is set to 90 degrees
- **THEN** the CSM cascade frusta match the 90-degree camera frustum, producing correct shadow coverage

#### Scenario: Coordinate system consistency
- **WHEN** the main camera uses `perspective_lh` projection
- **THEN** the CSM cascade matrices also use left-handed projection, eliminating shadow coordinate mismatch

### Requirement: Point Light and Spot Light Shadows

The system SHALL support shadow mapping for point lights (using cubemap shadow maps) and spot lights (using 2D perspective shadow maps).

Point light shadows SHALL use a 6-face cubemap with one depth-only render pass per face.

Spot light shadows SHALL use a single 2D depth texture with a perspective projection matching the spot cone angle.

#### Scenario: Point light shadow
- **WHEN** a point light is positioned inside a room
- **THEN** objects cast shadows in all directions from the light source

#### Scenario: Spot light shadow
- **WHEN** a spot light with a 45-degree cone angle illuminates a scene
- **THEN** shadows are cast only within the light's cone of influence

### Requirement: Configurable Backface Culling

The system SHALL support per-material backface culling configuration via the material system.

The default cull mode SHALL be `Back` for opaque materials and `None` for double-sided materials (as indicated by glTF `doubleSided` property).

#### Scenario: Single-sided mesh
- **WHEN** a closed mesh (e.g., sphere) is rendered with default cull mode
- **THEN** back faces are culled, improving rendering performance

#### Scenario: Double-sided material
- **WHEN** a glTF material has `doubleSided: true`
- **THEN** `cull_mode: None` is used, rendering both sides of each triangle
