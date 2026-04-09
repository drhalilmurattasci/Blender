/// Linear interpolation trait.
pub trait Lerp {
    /// Linearly interpolate between `self` and `other` by factor `t`.
    /// When `t == 0.0` returns `self`, when `t == 1.0` returns `other`.
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for f64 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_f32() {
        assert_eq!(0.0_f32.lerp(10.0, 0.0), 0.0);
        assert_eq!(0.0_f32.lerp(10.0, 1.0), 10.0);
        assert_eq!(0.0_f32.lerp(10.0, 0.5), 5.0);
    }

    #[test]
    fn test_lerp_f64() {
        assert_eq!(0.0_f64.lerp(10.0, 0.0), 0.0);
        assert_eq!(0.0_f64.lerp(10.0, 1.0), 10.0);
        assert_eq!(0.0_f64.lerp(10.0, 0.5), 5.0);
    }
}
