## ADDED Requirements

### Requirement: GPU Texture Creation
The system SHALL provide `create_texture()` for uploading RGBA image data to a GPU texture with an associated `TextureView`.

The system SHALL provide `create_sampler()` for creating texture samplers with configurable filtering modes.

#### Scenario: Texture upload
- **WHEN** `create_texture(device, queue, width, height, rgba_data, label)` is called
- **THEN** a wgpu `Texture` and `TextureView` are returned with the image data uploaded

#### Scenario: Sampler creation
- **WHEN** `create_sampler(device, label)` is called
- **THEN** a linear-filtering `Sampler` suitable for base color textures is returned

### Requirement: Multiple Bind Group Support
The system SHALL support binding multiple bind groups in a single render pass.

Group 0 SHALL be used for per-frame uniforms (MVP matrix). Group 1 SHALL be used for material data (textures and samplers).

#### Scenario: Two bind groups
- **WHEN** `set_bind_group(0, mvp_group)` and `set_bind_group(1, material_group)` are called in the render pass
- **THEN** both groups are accessible in the shader via `@group(0)` and `@group(1)`
