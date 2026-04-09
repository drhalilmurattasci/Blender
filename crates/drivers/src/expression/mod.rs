//! Simple expression evaluator for driver scripted expressions.

pub mod parser;
pub mod runtime;

pub use parser::parse_expression;
pub use runtime::{ExpressionContext, ExpressionValue};

/// A parsed driver expression ready for evaluation.
#[derive(Debug, Clone)]
pub struct Expression {
    /// The root node of the expression AST.
    pub root: ExprNode,
    /// The original expression string.
    pub source: String,
}

/// Expression AST node.
#[derive(Debug, Clone)]
pub enum ExprNode {
    /// Literal numeric constant.
    Literal(f64),
    /// Named variable reference.
    Variable(String),
    /// Binary operation.
    BinaryOp {
        op: BinaryOp,
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },
    /// Unary negation.
    Negate(Box<ExprNode>),
    /// Function call.
    FunctionCall {
        name: String,
        args: Vec<ExprNode>,
    },
    /// Ternary conditional: condition, then, else.
    Conditional {
        condition: Box<ExprNode>,
        then_expr: Box<ExprNode>,
        else_expr: Box<ExprNode>,
    },
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Equal,
    NotEqual,
    And,
    Or,
}
