//! Recursive-descent parser for driver expressions.
//!
//! Supports: arithmetic, comparisons, logical operators, function calls,
//! ternary conditionals, and variable references.

use crate::expression::{BinaryOp, ExprNode, Expression};
use crate::DriverError;

/// Parse an expression string into an `Expression`.
pub fn parse_expression(source: &str) -> Result<Expression, DriverError> {
    let tokens = tokenize(source)?;
    let mut parser = Parser::new(&tokens);
    let root = parser.parse_ternary()?;

    if parser.pos < parser.tokens.len() {
        return Err(DriverError::ParseError(format!(
            "unexpected token at position {}",
            parser.pos
        )));
    }

    Ok(Expression {
        root,
        source: source.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Comma,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    EqEq,
    NotEq,
    And,
    Or,
    Question,
    Colon,
}

fn tokenize(input: &str) -> Result<Vec<Token>, DriverError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => { tokens.push(Token::Minus); i += 1; }
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    tokens.push(Token::Caret);
                    i += 2;
                } else {
                    tokens.push(Token::Star);
                    i += 1;
                }
            }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '%' => { tokens.push(Token::Percent); i += 1; }
            '^' => { tokens.push(Token::Caret); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            ',' => { tokens.push(Token::Comma); i += 1; }
            '?' => { tokens.push(Token::Question); i += 1; }
            ':' => { tokens.push(Token::Colon); i += 1; }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::LessEq);
                    i += 2;
                } else {
                    tokens.push(Token::Less);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::GreaterEq);
                    i += 2;
                } else {
                    tokens.push(Token::Greater);
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::EqEq);
                    i += 2;
                } else {
                    return Err(DriverError::ParseError("unexpected '='".into()));
                }
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::NotEq);
                    i += 2;
                } else {
                    return Err(DriverError::ParseError("unexpected '!'".into()));
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::And);
                    i += 2;
                } else {
                    return Err(DriverError::ParseError("unexpected '&'".into()));
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token::Or);
                    i += 2;
                } else {
                    return Err(DriverError::ParseError("unexpected '|'".into()));
                }
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                // Handle scientific notation.
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                let val = s.parse::<f64>().map_err(|e| {
                    DriverError::ParseError(format!("invalid number `{s}`: {e}"))
                })?;
                tokens.push(Token::Number(val));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let ident: String = chars[start..i].iter().collect();
                // Handle keyword-style operators.
                match ident.as_str() {
                    "and" => tokens.push(Token::And),
                    "or" => tokens.push(Token::Or),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            c => {
                return Err(DriverError::ParseError(format!("unexpected character '{c}'")));
            }
        }
    }

    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), DriverError> {
        match self.advance() {
            Some(t) if t == expected => Ok(()),
            other => Err(DriverError::ParseError(format!(
                "expected {expected:?}, got {other:?}"
            ))),
        }
    }

    fn parse_ternary(&mut self) -> Result<ExprNode, DriverError> {
        let cond = self.parse_or()?;
        if self.peek() == Some(&Token::Question) {
            self.advance();
            let then_expr = self.parse_ternary()?;
            self.expect(&Token::Colon)?;
            let else_expr = self.parse_ternary()?;
            Ok(ExprNode::Conditional {
                condition: Box::new(cond),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        } else {
            Ok(cond)
        }
    }

    fn parse_or(&mut self) -> Result<ExprNode, DriverError> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = ExprNode::BinaryOp {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<ExprNode, DriverError> {
        let mut left = self.parse_comparison()?;
        while self.peek() == Some(&Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = ExprNode::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<ExprNode, DriverError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Some(Token::Less) => BinaryOp::Less,
                Some(Token::LessEq) => BinaryOp::LessEq,
                Some(Token::Greater) => BinaryOp::Greater,
                Some(Token::GreaterEq) => BinaryOp::GreaterEq,
                Some(Token::EqEq) => BinaryOp::Equal,
                Some(Token::NotEq) => BinaryOp::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = ExprNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<ExprNode, DriverError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = ExprNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<ExprNode, DriverError> {
        let mut left = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinaryOp::Mul,
                Some(Token::Slash) => BinaryOp::Div,
                Some(Token::Percent) => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_power()?;
            left = ExprNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_power(&mut self) -> Result<ExprNode, DriverError> {
        let base = self.parse_unary()?;
        if self.peek() == Some(&Token::Caret) {
            self.advance();
            let exp = self.parse_power()?; // Right-associative.
            Ok(ExprNode::BinaryOp {
                op: BinaryOp::Pow,
                left: Box::new(base),
                right: Box::new(exp),
            })
        } else {
            Ok(base)
        }
    }

    fn parse_unary(&mut self) -> Result<ExprNode, DriverError> {
        if self.peek() == Some(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(ExprNode::Negate(Box::new(expr)))
        } else if self.peek() == Some(&Token::Plus) {
            // Unary plus is a no-op but must be accepted.
            self.advance();
            self.parse_unary()
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<ExprNode, DriverError> {
        match self.advance().cloned() {
            Some(Token::Number(n)) => Ok(ExprNode::Literal(n)),
            Some(Token::Ident(name)) => {
                if self.peek() == Some(&Token::LParen) {
                    // Function call.
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_ternary()?);
                        while self.peek() == Some(&Token::Comma) {
                            self.advance();
                            args.push(self.parse_ternary()?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(ExprNode::FunctionCall { name, args })
                } else {
                    Ok(ExprNode::Variable(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_ternary()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            other => Err(DriverError::ParseError(format!(
                "unexpected token: {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::runtime::ExpressionContext;

    fn eval(source: &str) -> f64 {
        let expr = parse_expression(source).unwrap();
        ExpressionContext::new().evaluate(&expr).unwrap()
    }

    #[test]
    fn literal() {
        assert!((eval("42") - 42.0).abs() < 1e-12);
    }

    #[test]
    fn addition_subtraction() {
        assert!((eval("3 + 4 - 1") - 6.0).abs() < 1e-12);
    }

    #[test]
    fn mul_div_precedence() {
        // 2 + 3 * 4 = 14, not 20.
        assert!((eval("2 + 3 * 4") - 14.0).abs() < 1e-12);
    }

    #[test]
    fn parentheses_override_precedence() {
        assert!((eval("(2 + 3) * 4") - 20.0).abs() < 1e-12);
    }

    #[test]
    fn power_right_associative() {
        // 2 ^ 3 ^ 2 = 2 ^ 9 = 512
        assert!((eval("2 ^ 3 ^ 2") - 512.0).abs() < 1e-6);
    }

    #[test]
    fn unary_minus() {
        assert!((eval("-5") - -5.0).abs() < 1e-12);
        assert!((eval("--5") - 5.0).abs() < 1e-12);
    }

    #[test]
    fn unary_plus() {
        assert!((eval("+5") - 5.0).abs() < 1e-12);
        assert!((eval("+-5") - -5.0).abs() < 1e-12);
    }

    #[test]
    fn nested_function_calls() {
        assert!((eval("max(min(3, 5), 1)") - 3.0).abs() < 1e-12);
    }

    #[test]
    fn function_with_expression_args() {
        assert!((eval("max(1 + 2, 2 * 1)") - 3.0).abs() < 1e-12);
    }

    #[test]
    fn ternary_conditional() {
        assert!((eval("1 > 0 ? 10 : 20") - 10.0).abs() < 1e-12);
        assert!((eval("0 > 1 ? 10 : 20") - 20.0).abs() < 1e-12);
    }

    #[test]
    fn comparison_operators() {
        assert_eq!(eval("3 < 5"), 1.0);
        assert_eq!(eval("5 < 3"), 0.0);
        assert_eq!(eval("3 <= 3"), 1.0);
        assert_eq!(eval("3 >= 4"), 0.0);
        assert_eq!(eval("5 == 5"), 1.0);
        assert_eq!(eval("5 != 5"), 0.0);
    }

    #[test]
    fn logical_operators() {
        assert_eq!(eval("1 && 1"), 1.0);
        assert_eq!(eval("1 && 0"), 0.0);
        assert_eq!(eval("0 || 1"), 1.0);
        assert_eq!(eval("0 || 0"), 0.0);
    }

    #[test]
    fn keyword_operators() {
        assert_eq!(eval("1 and 1"), 1.0);
        assert_eq!(eval("0 or 1"), 1.0);
    }

    #[test]
    fn division_by_zero_returns_zero() {
        assert_eq!(eval("1 / 0"), 0.0);
    }

    #[test]
    fn modulo_by_zero_returns_zero() {
        assert_eq!(eval("5 % 0"), 0.0);
    }

    #[test]
    fn scientific_notation() {
        assert!((eval("1e3") - 1000.0).abs() < 1e-12);
        assert!((eval("1.5e-2") - 0.015).abs() < 1e-12);
    }

    #[test]
    fn double_star_power() {
        assert!((eval("2 ** 10") - 1024.0).abs() < 1e-6);
    }

    #[test]
    fn pi_and_e_constants() {
        assert!((eval("pi") - std::f64::consts::PI).abs() < 1e-10);
        assert!((eval("e") - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn sqrt_negative_no_nan() {
        // Should return 0.0, not NaN.
        let v = eval("sqrt(-4)");
        assert!(v.is_finite() && v >= 0.0, "sqrt(-4) = {v}");
    }

    #[test]
    fn asin_out_of_range_no_nan() {
        let v = eval("asin(2.0)");
        assert!(v.is_finite(), "asin(2.0) = {v}");
    }

    #[test]
    fn log_zero_not_nan() {
        let v = eval("log(0)");
        assert!(!v.is_nan(), "log(0) should not be NaN, got {v}");
    }
}
