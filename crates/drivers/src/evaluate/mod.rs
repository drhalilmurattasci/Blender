//! Driver evaluation: resolve variables and compute the final driven value.

use crate::expression::runtime::ExpressionContext;
use crate::expression::Expression;
use crate::variable::DriverVariable;
use crate::{DriverError, DriverResult, DriverType};

/// A fully configured driver attached to a property.
#[derive(Debug, Clone)]
pub struct Driver {
    /// How variables are combined.
    pub driver_type: DriverType,
    /// Variables that feed into the driver.
    pub variables: Vec<DriverVariable>,
    /// Scripted expression (used when `driver_type == ScriptedExpression`).
    pub expression: Option<Expression>,
    /// Whether this driver is muted.
    pub muted: bool,
    /// Whether this driver uses a self-reference (needs special ordering).
    pub use_self: bool,
}

impl Driver {
    /// Create a new driver with the given type.
    pub fn new(driver_type: DriverType) -> Self {
        Self {
            driver_type,
            variables: Vec::new(),
            expression: None,
            muted: false,
            use_self: false,
        }
    }

    /// Evaluate the driver, producing a final scalar value.
    ///
    /// Variable values must have been resolved externally before calling this
    /// (i.e., `cached_value` on each `DriverVariable` must be up to date).
    #[inline]
    pub fn evaluate(&self, frame: f64) -> DriverResult<f64> {
        if self.muted {
            return Ok(0.0);
        }

        match self.driver_type {
            DriverType::Average => self.evaluate_average(),
            DriverType::Sum => self.evaluate_sum(),
            DriverType::Min => self.evaluate_min(),
            DriverType::Max => self.evaluate_max(),
            DriverType::ScriptedExpression => self.evaluate_expression(frame),
        }
    }

    fn evaluate_average(&self) -> DriverResult<f64> {
        if self.variables.is_empty() {
            return Ok(0.0);
        }
        let sum: f64 = self.variables.iter().map(|v| v.cached_value).sum();
        Ok(sum / self.variables.len() as f64)
    }

    fn evaluate_sum(&self) -> DriverResult<f64> {
        Ok(self.variables.iter().map(|v| v.cached_value).sum())
    }

    fn evaluate_min(&self) -> DriverResult<f64> {
        self.variables
            .iter()
            .map(|v| v.cached_value)
            .reduce(f64::min)
            .ok_or_else(|| DriverError::EvalError("no variables for min".into()))
    }

    fn evaluate_max(&self) -> DriverResult<f64> {
        self.variables
            .iter()
            .map(|v| v.cached_value)
            .reduce(f64::max)
            .ok_or_else(|| DriverError::EvalError("no variables for max".into()))
    }

    fn evaluate_expression(&self, frame: f64) -> DriverResult<f64> {
        let expr = self
            .expression
            .as_ref()
            .ok_or_else(|| DriverError::EvalError("no expression set".into()))?;

        let mut ctx = ExpressionContext::new();
        ctx.frame = frame;
        for var in &self.variables {
            ctx.variables.insert(var.name.clone(), var.cached_value);
        }

        ctx.evaluate(expr)
    }
}
