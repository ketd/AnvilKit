## ADDED Requirements

### Requirement: Blinn-Phong Lighting
The system SHALL demonstrate Blinn-Phong lighting capability through example shaders that combine ambient, diffuse (Lambert), and specular (Blinn-Phong) components.

The lighting model SHALL operate in world space using separated model/view/projection matrices and a proper normal matrix (inverse transpose of model matrix).

#### Scenario: Lit textured model
- **WHEN** a textured model is rendered with a directional light
- **THEN** the model shows ambient illumination on shadowed faces, diffuse shading proportional to surface-to-light angle, and specular highlights near the reflection direction

#### Scenario: Normal matrix correctness
- **WHEN** a model has non-uniform scaling
- **THEN** normals are correctly transformed using the inverse-transpose of the model matrix, preventing shading distortion
