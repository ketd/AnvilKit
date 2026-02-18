## ADDED Requirements

### Requirement: Cook-Torrance PBR BRDF
The system SHALL demonstrate physically-based rendering using the Cook-Torrance microfacet BRDF model with:
- GGX/Trowbridge-Reitz Normal Distribution Function (NDF)
- Schlick approximation for Fresnel reflectance
- Smith GGX geometric attenuation (height-correlated)

The BRDF SHALL use metallic-roughness workflow parameters from glTF materials.

#### Scenario: Metallic vs dielectric
- **WHEN** a metallic sphere (metallic=1.0, roughness=0.3) and a dielectric sphere (metallic=0.0, roughness=0.3) are rendered under the same light
- **THEN** the metallic sphere shows tinted reflections using base color as F0, while the dielectric shows white reflections with F0=0.04

#### Scenario: Roughness variation
- **WHEN** surfaces with roughness from 0.0 to 1.0 are rendered
- **THEN** roughness=0.0 produces sharp specular highlights and roughness=1.0 produces wide diffuse-like reflections

### Requirement: HDR Rendering Pipeline
The system SHALL support high dynamic range rendering through:
- Rgba16Float offscreen render target for scene rendering
- Full-screen post-processing pass for tone mapping
- ACES Filmic tone mapping operator
- Linear-space rendering with sRGB output conversion

The system SHALL support multi-pass rendering (scene pass → post-process pass) in `RenderApp`.

#### Scenario: HDR to LDR conversion
- **WHEN** a scene with bright specular highlights (values > 1.0) is rendered
- **THEN** the tone mapping compresses the dynamic range to [0,1] without clipping, preserving highlight detail

#### Scenario: Multi-pass rendering
- **WHEN** the scene pass renders to an offscreen HDR target
- **THEN** the post-process pass reads the HDR texture and outputs tone-mapped results to the swap chain

### Requirement: Image-Based Lighting (IBL)
The system SHALL support environment lighting through:
- HDR equirectangular environment map loading (.hdr format)
- Equirectangular to cubemap conversion (GPU-based)
- Diffuse irradiance map convolution
- Specular prefiltered environment map (split-sum approximation, multiple mip levels)
- BRDF integration LUT (2D lookup texture)

The ambient lighting term SHALL combine diffuse IBL (irradiance * albedo) and specular IBL (prefiltered env * BRDF LUT).

#### Scenario: Environment reflection
- **WHEN** a metallic sphere (metallic=1.0, roughness=0.0) is rendered with an HDR environment map
- **THEN** the sphere shows mirror-like reflections of the environment

#### Scenario: Diffuse environment lighting
- **WHEN** a dielectric sphere (metallic=0.0) is rendered with an HDR environment map
- **THEN** the sphere is lit by the environment's average color from all directions (irradiance)

### Requirement: Normal Mapping
The system SHALL support tangent-space normal mapping through:
- Extended vertex format with tangent attribute `[f32; 4]` (xyz = tangent direction, w = bitangent sign)
- TBN (Tangent-Bitangent-Normal) matrix construction in vertex shader
- Normal map sampling and world-space normal perturbation in fragment shader

The system SHALL extract tangent data from glTF files or compute it using MikkTSpace algorithm.

#### Scenario: Surface detail
- **WHEN** a flat surface with a brick normal map is rendered
- **THEN** the surface appears to have 3D brick geometry despite being geometrically flat

#### Scenario: Tangent extraction
- **WHEN** a glTF file contains TANGENT attributes
- **THEN** they are loaded directly; otherwise tangents are computed from positions, normals, and UVs

### Requirement: Complete Material System
The system SHALL support the full glTF PBR material model including:
- Base color texture + factor
- Metallic-roughness texture + factors
- Normal map with configurable scale
- Ambient occlusion map
- Emissive texture + factor

The system SHALL support multiple simultaneous light sources (directional lights + point lights).

#### Scenario: Full material rendering
- **WHEN** a glTF model with all PBR textures (baseColor, metallicRoughness, normal, AO, emissive) is loaded
- **THEN** all texture channels contribute to the final rendered appearance

#### Scenario: Multiple lights
- **WHEN** a scene contains both directional and point lights
- **THEN** each light contributes independently to the final illumination using the PBR BRDF
