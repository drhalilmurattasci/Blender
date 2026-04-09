//! # forge3d-drivers
//!
//! Animation drivers: expression-based property linking system.
//!
//! Drivers allow one property to be controlled by an expression that
//! references other properties, transforms, distances, etc.

pub mod evaluate;
pub mod expression;
pub mod source;
pub mod variable;

use thiserror::Error;

/// Errors from driver evaluation.
#[derive(Debug, Error)]
pub enum DriverError {
    #[error("variable `{0}` not found")]
    VariableNotFound(String),

    #[error("expression parse error: {0}")]
    ParseError(String),

    #[error("expression evaluation error: {0}")]
    EvalError(String),

    #[error("cyclic driver dependency detected")]
    CyclicDependency,

    #[error("invalid driver target: {0}")]
    InvalidTarget(String),
}

pub type DriverResult<T> = Result<T, DriverError>;

/// The type of driver expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriverType {
    /// Average all variable values.
    Average,
    /// Sum all variable values.
    Sum,
    /// Use a scripted expression.
    ScriptedExpression,
    /// Use the minimum variable value.
    Min,
    /// Use the maximum variable value.
    Max,
}

impl Default for DriverType {
    fn default() -> Self {
        Self::Average
    }
}
