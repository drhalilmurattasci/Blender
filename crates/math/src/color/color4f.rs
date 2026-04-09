use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::traits::{ApproxEq, Lerp};

/// RGBA color with f32 components in linear space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// SAFETY: repr(C) with only f32 fields.
unsafe impl Zeroable for Color4f {}
unsafe impl Pod for Color4f {}

impl Color4f {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Convert a single component from sRGB to linear.
    ///
    /// Uses the IEC 61966-2-1 standard (same as Blender): linear toe region
    /// below 0.04045, and gamma 2.4 above. Negative values are mirrored.
    #[inline]
    fn srgb_to_linear_component(c: f32) -> f32 {
        if c < 0.0 {
            return -Self::srgb_to_linear_component(-c);
        }
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Convert a single component from linear to sRGB.
    ///
    /// Uses the IEC 61966-2-1 standard (same as Blender): linear toe region
    /// below 0.0031308, and gamma 1/2.4 above. Negative values are mirrored.
    #[inline]
    fn linear_to_srgb_component(c: f32) -> f32 {
        if c < 0.0 {
            return -Self::linear_to_srgb_component(-c);
        }
        if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    /// Create from sRGB color space (components in 0..1). Alpha is unchanged.
    #[inline]
    pub fn from_srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: Self::srgb_to_linear_component(r),
            g: Self::srgb_to_linear_component(g),
            b: Self::srgb_to_linear_component(b),
            a,
        }
    }

    /// Create from sRGB u8 values (0..255).
    #[inline]
    pub fn from_srgb_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_srgb(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Convert to sRGB color space. Alpha is unchanged.
    #[inline]
    pub fn to_srgb(self) -> [f32; 4] {
        [
            Self::linear_to_srgb_component(self.r),
            Self::linear_to_srgb_component(self.g),
            Self::linear_to_srgb_component(self.b),
            self.a,
        ]
    }

    /// Convert to sRGB u8 values (0..255).
    #[inline]
    pub fn to_srgb_u8(self) -> [u8; 4] {
        let srgb = self.to_srgb();
        [
            (srgb[0] * 255.0 + 0.5) as u8,
            (srgb[1] * 255.0 + 0.5) as u8,
            (srgb[2] * 255.0 + 0.5) as u8,
            (srgb[3] * 255.0 + 0.5) as u8,
        ]
    }

    /// Convert to a `Color3f` by dropping the alpha channel.
    #[inline]
    pub fn to_rgb(self) -> crate::color::Color3f {
        crate::color::Color3f::new(self.r, self.g, self.b)
    }

    /// Convert to array.
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color4f {
    #[inline]
    fn default() -> Self {
        Self::BLACK
    }
}

impl From<[f32; 4]> for Color4f {
    #[inline]
    fn from(a: [f32; 4]) -> Self {
        Self { r: a[0], g: a[1], b: a[2], a: a[3] }
    }
}

impl From<Color4f> for [f32; 4] {
    #[inline]
    fn from(c: Color4f) -> Self {
        c.to_array()
    }
}

impl Lerp for Color4f {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

impl ApproxEq for Color4f {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self.r - other.r).abs() <= epsilon
            && (self.g - other.g).abs() <= epsilon
            && (self.b - other.b).abs() <= epsilon
            && (self.a - other.a).abs() <= epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_roundtrip() {
        let c = Color4f::from_srgb(0.5, 0.5, 0.5, 1.0);
        let srgb = c.to_srgb();
        assert!((srgb[0] - 0.5).abs() < 1e-4);
        assert!((srgb[1] - 0.5).abs() < 1e-4);
        assert!((srgb[2] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_pod() {
        let c = Color4f::WHITE;
        let bytes = bytemuck::bytes_of(&c);
        assert_eq!(bytes.len(), 16);
    }
}
