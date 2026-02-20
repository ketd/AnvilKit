use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnvilKitConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub render: RenderConfig,
    #[serde(default)]
    pub camera: CameraConfig,
    #[serde(default)]
    pub physics: PhysicsConfig,
    #[serde(default)]
    pub scene: SceneConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub package: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AnvilKit Game".into(),
            width: 1280,
            height: 720,
            vsync: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub msaa_samples: u32,
    pub hdr: bool,
    pub shadows: bool,
    pub clear_color: [f32; 4],
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            msaa_samples: 4,
            hdr: true,
            shadows: true,
            clear_color: [0.15, 0.3, 0.6, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub position: [f32; 3],
    pub look_at: [f32; 3],
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            fov: 55.0,
            near: 0.1,
            far: 100.0,
            position: [0.0, 12.0, -10.0],
            look_at: [0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub gravity: [f32; 3],
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gravity: [0.0, -9.81, 0.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    #[serde(default)]
    pub lighting: LightingConfig,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            lighting: LightingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingConfig {
    pub directional_direction: [f32; 3],
    pub directional_color: [f32; 3],
    pub directional_intensity: f32,
    #[serde(default)]
    pub point_lights: Vec<PointLightConfig>,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            directional_direction: [-0.4, -0.7, 0.5],
            directional_color: [1.0, 0.95, 0.85],
            directional_intensity: 4.0,
            point_lights: vec![PointLightConfig {
                position: [3.0, 4.0, 0.0],
                color: [1.0, 0.8, 0.6],
                intensity: 15.0,
                range: 15.0,
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLightConfig {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
}
