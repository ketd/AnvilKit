## ADDED Requirements

### Requirement: Texture Data Extraction
The system SHALL provide a `TextureData` struct containing image width, height, and RGBA pixel data extracted from glTF files.

The system SHALL support both embedded glTF textures (base64/binary) and external image references.

#### Scenario: Embedded texture extraction
- **WHEN** a glTF file contains an embedded base color texture
- **THEN** `TextureData` is returned with correct dimensions and RGBA pixel data

#### Scenario: No texture available
- **WHEN** a glTF material has no base color texture
- **THEN** the material's `base_color_texture` field is `None` and `base_color_factor` provides a fallback color

### Requirement: Material Data Extraction
The system SHALL provide a `MaterialData` struct containing base color texture reference (optional) and base color factor `[f32; 4]`.

#### Scenario: Material with texture
- **WHEN** a glTF primitive has a material with a base color texture
- **THEN** `MaterialData` contains both the texture reference and the color factor

#### Scenario: Material without texture
- **WHEN** a glTF material has only a base color factor (no texture)
- **THEN** `MaterialData.base_color_texture` is `None` and `base_color_factor` contains the RGBA color
