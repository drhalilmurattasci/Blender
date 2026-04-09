//! Common error and result types for mesh operations.

use forge3d_mesh::MeshError;
use thiserror::Error;

/// Errors that can occur during mesh editing operations.
#[derive(Debug, Error)]
pub enum OpError {
    /// An error propagated from the core mesh layer.
    #[error(transparent)]
    Mesh(#[from] MeshError),

    /// The operation received invalid parameters.
    #[error("invalid parameter: {0}")]
    InvalidParam(String),

    /// The selection is empty or incompatible with the operation.
    #[error("invalid selection: {0}")]
    InvalidSelection(String),

    /// The operation is not yet implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),

    /// A geometric computation failed (e.g. degenerate polygon).
    #[error("geometry error: {0}")]
    GeometryError(String),
}

/// Convenience alias for operation results.
pub type OpResult<T = ()> = Result<T, OpError>;
