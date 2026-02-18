# Change: Update Project Roadmap and Render System Spec for M6a+

## Why
M6a (ECS PBR unification + legacy cleanup) was completed but the project roadmap and render-system spec still reference deleted types (RenderContext, MeshComponent, MaterialComponent, RenderComponent, RenderSystemSet, set_pipeline_3d) and lack documentation for new types (PbrSceneUniform, SceneLights, MaterialParams, DirectionalLight). The roadmap M5-M10 is vaguely described and needs detailed breakdown through M12 for a complete game engine plan.

## What Changes
- **REMOVED** render-system requirements for deleted code: Render Context, legacy pipeline setters, legacy components
- **MODIFIED** render-system requirements to reflect ECS-only architecture: plugin components, uniform buffer (256-byte PBR), bind group layout, PBR BRDF delivery mechanism
- **ADDED** render-system requirements for new M6a types: PbrSceneUniform, SceneLights, MaterialParams
- **Updated** project.md Milestone Roadmap from vague M5-M10 to detailed M0-M12 covering rendering quality, production rendering, performance, scene infrastructure, game systems, advanced features, and developer experience

## Impact
- Affected specs: render-system
- Affected code: None (documentation-only change)
- No code changes required — this change synchronizes specs and roadmap with already-shipped M6a code
