## ADDED Requirements

### Requirement: HDR Environment Map Loading
The system SHALL provide loading of HDR equirectangular environment maps in Radiance HDR (.hdr) format.

The loader SHALL return floating-point RGB pixel data suitable for GPU texture creation.

#### Scenario: HDR file loading
- **WHEN** `load_hdr_environment("assets/env.hdr")` is called with a valid HDR file
- **THEN** floating-point RGB data with width, height, and pixel values potentially exceeding 1.0 are returned

### Requirement: Complete glTF Material Extraction
The system SHALL extract the full PBR material definition from glTF files including:
- metallic factor and roughness factor
- metallic-roughness texture (if present)
- normal map texture with scale
- occlusion texture
- emissive texture and factor

#### Scenario: Full material extraction
- **WHEN** a glTF file contains a material with metallicRoughness texture and normal map
- **THEN** `MaterialData` contains both textures and all scalar parameters

## MODIFIED Requirements

### Requirement: Material Data Extraction
The system SHALL provide a `MaterialData` struct containing base color texture reference (optional), base color factor `[f32; 4]`, metallic factor `f32`, roughness factor `f32`, normal map texture (optional), normal scale `f32`, occlusion texture (optional), emissive texture (optional), and emissive factor `[f32; 3]`.

#### Scenario: Material with texture
- **WHEN** a glTF primitive has a material with a base color texture
- **THEN** `MaterialData` contains both the texture reference and the color factor

#### Scenario: Material without texture
- **WHEN** a glTF material has only a base color factor (no texture)
- **THEN** `MaterialData.base_color_texture` is `None` and `base_color_factor` contains the RGBA color

#### Scenario: PBR parameters
- **WHEN** a glTF material specifies metallic=0.8 and roughness=0.2
- **THEN** `MaterialData.metallic_factor` is 0.8 and `MaterialData.roughness_factor` is 0.2
