/// Approximate equality comparison trait.
pub trait ApproxEq {
    /// Default epsilon for comparison.
    const DEFAULT_EPSILON: f32 = 1e-6;

    /// Returns `true` if `self` and `other` are approximately equal
    /// within the given `epsilon`.
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool;

    /// Returns `true` if `self` and `other` are approximately equal
    /// using the default epsilon.
    #[inline]
    fn approx_eq_default(&self, other: &Self) -> bool {
        self.approx_eq(other, Self::DEFAULT_EPSILON)
    }
}

impl ApproxEq for f32 {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self - other).abs() <= epsilon
    }
}

impl ApproxEq for f64 {
    #[inline]
    fn approx_eq(&self, other: &Self, epsilon: f32) -> bool {
        (self - other).abs() <= epsilon as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_eq_f32() {
        assert!(1.0_f32.approx_eq(&1.0000001, 1e-6));
        assert!(!1.0_f32.approx_eq(&1.01, 1e-6));
    }

    #[test]
    fn test_approx_eq_f64() {
        assert!(1.0_f64.approx_eq(&1.0000001, 1e-6));
        assert!(!1.0_f64.approx_eq(&1.01, 1e-6));
    }
}
