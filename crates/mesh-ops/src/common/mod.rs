//! Common types and utilities shared across mesh operations.

pub mod result;
pub mod selection;

pub use result::{OpError, OpResult};
pub use selection::SelectionHelper;
