//! Noise modifier: adds procedural noise to F-Curve values.

use serde::{Deserialize, Serialize};

/// Noise modifier configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseModifier {
    /// Blend mode: how noise combines with the base value.
    pub blend_type: NoiseBlendType,
    /// Noise amplitude (strength).
    pub strength: f32,
    /// Scale factor for time (affects frequency).
    pub scale: f32,
    /// Phase offset for the noise.
    pub phase: f32,
    /// Depth of noise octaves for fractal noise.
    pub depth: u32,
    /// Offset applied to the noise output.
    pub offset: f32,
}

/// How noise is blended with the base curve value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NoiseBlendType {
    /// Replace the base value with noise.
    Replace,
    /// Add noise to the base value.
    Add,
    /// Subtract noise from the base value.
    Subtract,
    /// Multiply the base value by noise.
    Multiply,
}

impl Default for NoiseBlendType {
    fn default() -> Self {
        Self::Add
    }
}

impl Default for NoiseModifier {
    fn default() -> Self {
        Self {
            blend_type: NoiseBlendType::Add,
            strength: 1.0,
            scale: 1.0,
            phase: 0.0,
            depth: 0,
            offset: 0.0,
        }
    }
}

impl NoiseModifier {
    /// Apply the noise modifier to a base value at the given time.
    pub fn apply(&self, time: f32, value: f32) -> f32 {
        let noise_val = self.evaluate_noise(time);

        match self.blend_type {
            NoiseBlendType::Replace => noise_val,
            NoiseBlendType::Add => value + noise_val,
            NoiseBlendType::Subtract => value - noise_val,
            NoiseBlendType::Multiply => value * noise_val,
        }
    }

    /// Simple hash-based noise (deterministic, no external dependency).
    fn evaluate_noise(&self, time: f32) -> f32 {
        let t = (time + self.phase) * self.scale;
        let mut amplitude = self.strength;
        let mut result = 0.0;
        let mut freq = 1.0;

        for _ in 0..=self.depth {
            result += self.noise_1d(t * freq) * amplitude;
            freq *= 2.0;
            amplitude *= 0.5;
        }

        result + self.offset
    }

    /// Simple 1D value noise using integer hashing and smooth interpolation.
    fn noise_1d(&self, t: f32) -> f32 {
        let i = t.floor() as i32;
        let f = t - t.floor();
        // Smoothstep.
        let u = f * f * (3.0 - 2.0 * f);

        let a = hash_f32(i);
        let b = hash_f32(i.wrapping_add(1));

        a + (b - a) * u
    }
}

/// Simple integer hash mapped to `[-1, 1]`.
fn hash_f32(n: i32) -> f32 {
    let n = (n as u32).wrapping_mul(0x45d9f3b).wrapping_add(0x1234567);
    let n = n ^ (n >> 16);
    let n = n.wrapping_mul(0x45d9f3b);
    // Map to [-1, 1].
    (n as f32 / u32::MAX as f32) * 2.0 - 1.0
}
