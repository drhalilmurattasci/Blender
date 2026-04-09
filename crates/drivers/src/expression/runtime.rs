//! Expression runtime: evaluates parsed expression ASTs.

use crate::expression::{BinaryOp, ExprNode, Expression};
use crate::DriverError;
use std::collections::HashMap;

/// A value produced by expression evaluation.
pub type ExpressionValue = f64;

/// Context providing variable values and built-in functions to the expression evaluator.
pub struct ExpressionContext {
    /// Variable name -> value mapping.
    pub variables: HashMap<String, f64>,
    /// Current frame (available as `frame`).
    pub frame: f64,
}

impl ExpressionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            frame: 0.0,
        }
    }

    /// Evaluate an expression in this context.
    pub fn evaluate(&self, expr: &Expression) -> Result<f64, DriverError> {
        self.eval_node(&expr.root)
    }

    fn eval_node(&self, node: &ExprNode) -> Result<f64, DriverError> {
        match node {
            ExprNode::Literal(v) => Ok(*v),

            ExprNode::Variable(name) => {
                if name == "frame" {
                    return Ok(self.frame);
                }
                if name == "pi" {
                    return Ok(std::f64::consts::PI);
                }
                if name == "e" {
                    return Ok(std::f64::consts::E);
                }
                self.variables
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| DriverError::VariableNotFound(name.clone()))
            }

            ExprNode::BinaryOp { op, left, right } => {
                let l = self.eval_node(left)?;
                let r = self.eval_node(right)?;
                Ok(eval_binary_op(*op, l, r))
            }

            ExprNode::Negate(inner) => {
                let v = self.eval_node(inner)?;
                Ok(-v)
            }

            ExprNode::FunctionCall { name, args } => {
                let evaluated_args: Result<Vec<f64>, _> =
                    args.iter().map(|a| self.eval_node(a)).collect();
                let args = evaluated_args?;
                eval_function(name, &args)
            }

            ExprNode::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond = self.eval_node(condition)?;
                if cond != 0.0 {
                    self.eval_node(then_expr)
                } else {
                    self.eval_node(else_expr)
                }
            }
        }
    }
}

impl Default for ExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

fn eval_binary_op(op: BinaryOp, l: f64, r: f64) -> f64 {
    match op {
        BinaryOp::Add => l + r,
        BinaryOp::Sub => l - r,
        BinaryOp::Mul => l * r,
        BinaryOp::Div => {
            if r.abs() < f64::EPSILON {
                0.0
            } else {
                l / r
            }
        }
        BinaryOp::Mod => {
            if r.abs() < f64::EPSILON {
                0.0
            } else {
                l % r
            }
        }
        BinaryOp::Pow => l.powf(r),
        BinaryOp::Less => if l < r { 1.0 } else { 0.0 },
        BinaryOp::LessEq => if l <= r { 1.0 } else { 0.0 },
        BinaryOp::Greater => if l > r { 1.0 } else { 0.0 },
        BinaryOp::GreaterEq => if l >= r { 1.0 } else { 0.0 },
        BinaryOp::Equal => if (l - r).abs() < f64::EPSILON { 1.0 } else { 0.0 },
        BinaryOp::NotEqual => if (l - r).abs() >= f64::EPSILON { 1.0 } else { 0.0 },
        BinaryOp::And => if l != 0.0 && r != 0.0 { 1.0 } else { 0.0 },
        BinaryOp::Or => if l != 0.0 || r != 0.0 { 1.0 } else { 0.0 },
    }
}

fn eval_function(name: &str, args: &[f64]) -> Result<f64, DriverError> {
    let err = |msg: &str| DriverError::EvalError(format!("{}(): {}", name, msg));

    match name {
        "sin" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.sin())
        }
        "cos" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.cos())
        }
        "tan" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.tan())
        }
        "asin" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.clamp(-1.0, 1.0).asin())
        }
        "acos" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.clamp(-1.0, 1.0).acos())
        }
        "atan" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.atan())
        }
        "atan2" => {
            if args.len() < 2 { return Err(err("expected 2 arguments")); }
            Ok(args[0].atan2(args[1]))
        }
        "sqrt" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.max(0.0).sqrt())
        }
        "abs" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.abs())
        }
        "floor" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.floor())
        }
        "ceil" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.ceil())
        }
        "round" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.round())
        }
        "min" => {
            if args.len() < 2 { return Err(err("expected 2 arguments")); }
            Ok(args[0].min(args[1]))
        }
        "max" => {
            if args.len() < 2 { return Err(err("expected 2 arguments")); }
            Ok(args[0].max(args[1]))
        }
        "clamp" => {
            if args.len() < 3 { return Err(err("expected 3 arguments")); }
            Ok(args[0].clamp(args[1], args[2]))
        }
        "lerp" => {
            if args.len() < 3 { return Err(err("expected 3 arguments")); }
            Ok(args[0] + (args[1] - args[0]) * args[2])
        }
        "log" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            if *a <= 0.0 {
                Ok(f64::NEG_INFINITY)
            } else {
                Ok(a.ln())
            }
        }
        "log10" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            if *a <= 0.0 {
                Ok(f64::NEG_INFINITY)
            } else {
                Ok(a.log10())
            }
        }
        "exp" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.exp())
        }
        "pow" => {
            if args.len() < 2 { return Err(err("expected 2 arguments")); }
            Ok(args[0].powf(args[1]))
        }
        "radians" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.to_radians())
        }
        "degrees" => {
            let a = args.first().ok_or_else(|| err("expected 1 argument"))?;
            Ok(a.to_degrees())
        }
        _ => Err(DriverError::EvalError(format!("unknown function `{name}`"))),
    }
}
