## ADDED Requirements

### Requirement: Window Management
The system SHALL provide `WindowConfig` for configuring window properties (title, size, resizable, fullscreen) using a builder pattern.

The system SHALL provide `RenderApp` implementing winit's event handling for window lifecycle management (create, resize, close, input events).

#### Scenario: Window creation
- **WHEN** `WindowConfig::new().with_title("Game").with_size(1280, 720)` is used to create a window
- **THEN** a platform window with the specified title and dimensions is created

#### Scenario: Window resize handling
- **WHEN** the user resizes the window
- **THEN** the render surface is reconfigured to match the new dimensions

### Requirement: GPU Device Management
The system SHALL provide `RenderDevice` wrapping wgpu `Instance`, `Adapter`, `Device`, and `Queue`.

The system SHALL support automatic GPU adapter selection with fallback to software rendering.

#### Scenario: Device initialization
- **WHEN** `RenderDevice` is created
- **THEN** a compatible GPU adapter is selected and a device/queue pair is obtained

#### Scenario: No GPU available
- **WHEN** no compatible GPU adapter is found
- **THEN** the system returns an appropriate error with a clear message

### Requirement: Render Surface Management
The system SHALL provide `RenderSurface` managing the wgpu swap chain surface, including format selection, configuration, and frame acquisition.

#### Scenario: Surface configuration
- **WHEN** a render surface is created for a window
- **THEN** a compatible texture format is selected and the surface is configured

#### Scenario: Frame acquisition
- **WHEN** a new frame is requested
- **THEN** a `SurfaceTexture` and `TextureView` are returned for rendering

### Requirement: Render Context
The system SHALL provide `RenderContext` as a unified high-level rendering interface combining device and surface management.

#### Scenario: Begin and end frame
- **WHEN** a frame render cycle begins
- **THEN** the context acquires a surface texture, creates a command encoder, and provides a render pass

### Requirement: Render Pipeline Builder
The system SHALL provide `RenderPipelineBuilder` with a fluent API for constructing wgpu render pipelines, including shader module creation from WGSL source.

The system SHALL provide `BasicRenderPipeline` wrapping a configured wgpu pipeline.

#### Scenario: Pipeline creation
- **WHEN** a pipeline is built with vertex shader, fragment shader, and vertex format
- **THEN** a valid `RenderPipeline` is created and ready for use in render passes

### Requirement: ECS Render Plugin
The system SHALL provide `RenderPlugin` implementing the ECS `Plugin` trait to register rendering systems and resources.

The system SHALL provide rendering components:
- `RenderComponent` — marks an entity for rendering (visible flag, layer)
- `CameraComponent` — camera parameters (FOV, near/far planes, active flag)
- `MeshComponent` — mesh geometry reference (mesh_id, vertex/index counts)
- `MaterialComponent` — material properties (base color, metallic, roughness)

The system SHALL provide `RenderConfig` as a global resource and `RenderSystemSet` for system ordering.

#### Scenario: Plugin registration
- **WHEN** `app.add_plugins(RenderPlugin)` is called
- **THEN** rendering systems and resources are registered in the ECS world

#### Scenario: Render component query
- **WHEN** a system queries for `(Entity, &RenderComponent, &CameraComponent)`
- **THEN** only entities with both components are returned
