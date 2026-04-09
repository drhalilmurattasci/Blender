//! Subdivision operations.

pub mod catmull_clark;
pub mod simple;

pub use catmull_clark::catmull_clark;
pub use simple::subdivide_simple;

/// Parameters for simple subdivision.
#[derive(Debug, Clone)]
pub struct SubdivideParams {
    /// Number of subdivision iterations.
    pub iterations: u32,
}

impl Default for SubdivideParams {
    fn default() -> Self {
        Self { iterations: 1 }
    }
}

/// Parameters for Catmull-Clark subdivision.
#[derive(Debug, Clone)]
pub struct CatmullClarkParams {
    /// Number of subdivision iterations.
    pub iterations: u32,
    /// Whether to apply smooth shading to the result.
    pub smooth: bool,
}

impl Default for CatmullClarkParams {
    fn default() -> Self {
        Self {
            iterations: 1,
            smooth: true,
        }
    }
}
