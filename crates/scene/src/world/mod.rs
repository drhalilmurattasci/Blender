//! World / environment settings: background color, ambient light, and
//! environment map references.

use serde::{Deserialize, Serialize};

/// Ambient light contribution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AmbientLight {
    /// Linear-space RGB color.
    pub color: [f32; 3],
    /// Intensity multiplier.
    pub energy: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: [0.05, 0.05, 0.05],
            energy: 1.0,
        }
    }
}

/// Type of environment background.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackgroundType {
    /// Solid color.
    Color,
    /// Equirectangular HDR image (referenced by path / asset id).
    EnvironmentMap,
    /// Procedural sky model.
    Sky,
}

impl Default for BackgroundType {
    fn default() -> Self {
        Self::Color
    }
}

/// World / environment settings for a scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    /// Background type selector.
    pub background_type: BackgroundType,
    /// Solid background color (used when `background_type == Color`).
    pub background_color: [f32; 3],
    /// Optional path or asset reference for the environment map.
    pub environment_map: Option<String>,
    /// Ambient light.
    pub ambient: AmbientLight,
    /// Mist / fog start distance.
    pub mist_start: f32,
    /// Mist / fog depth (distance over which opacity goes 0..1).
    pub mist_depth: f32,
    /// Whether mist is enabled.
    pub mist_enabled: bool,
}

impl Default for World {
    fn default() -> Self {
        Self {
            background_type: BackgroundType::default(),
            background_color: [0.05, 0.05, 0.05],
            environment_map: None,
            ambient: AmbientLight::default(),
            mist_start: 5.0,
            mist_depth: 25.0,
            mist_enabled: false,
        }
    }
}
