use super::token::Token;
use crate::cell::{CellCoordinate, CellRange, CellValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Power,
    Concat,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormulaExpr {
    Literal(CellValue),
    Cell(CellCoordinate),
    Range(CellRange),
    Spill(CellCoordinate),
    Unary(UnaryOp, Box<FormulaExpr>),
    Binary(BinaryOp, Box<FormulaExpr>, Box<FormulaExpr>),
    FunctionCall { name: String, args: Vec<FormulaExpr> },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    pub fn parse(&mut self) -> Result<FormulaExpr, String> {
        self.parse_expr(0)
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<FormulaExpr, String> {
        let mut lhs = self.parse_primary()?;

        while let Some(tok) = self.peek() {
            if let Some((l_bp, r_bp, op)) = Self::infix_binding_power(tok) {
                if l_bp < min_bp {
                    break;
                }
                self.next_token();
                let rhs = self.parse_expr(r_bp)?;
                lhs = FormulaExpr::Binary(op, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<FormulaExpr, String> {
        let tok = self.next_token().ok_or("Unexpected end of formula")?;
        match tok {
            Token::Number(n) => Ok(FormulaExpr::Literal(CellValue::Number(n))),
            Token::StringLiteral(s) => Ok(FormulaExpr::Literal(CellValue::Text(s))),
            Token::Boolean(b) => Ok(FormulaExpr::Literal(CellValue::Boolean(b))),
            Token::CellRef(c) => Ok(FormulaExpr::Cell(c)),
            Token::CellRangeRef { start, end } => Ok(FormulaExpr::Range(CellRange::new(start, end))),
            Token::SpillRef(c) => Ok(FormulaExpr::Spill(c)),
            Token::Minus => {
                let rhs = self.parse_expr(60)?;
                Ok(FormulaExpr::Unary(UnaryOp::Neg, Box::new(rhs)))
            }
            Token::Plus => {
                let rhs = self.parse_expr(60)?;
                Ok(FormulaExpr::Unary(UnaryOp::Pos, Box::new(rhs)))
            }
            Token::OpenParen => {
                let expr = self.parse_expr(0)?;
                if self.next_token() != Some(Token::CloseParen) {
                    return Err("Expected closing parenthesis ')'".into());
                }
                Ok(expr)
            }
            Token::Ident(name) => {
                if self.peek() == Some(&Token::OpenParen) {
                    self.next_token(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::CloseParen) {
                        loop {
                            args.push(self.parse_expr(0)?);
                            if self.peek() == Some(&Token::Comma) {
                                self.next_token();
                            } else {
                                break;
                            }
                        }
                    }
                    if self.next_token() != Some(Token::CloseParen) {
                        return Err(format!("Expected ')' after function {}", name));
                    }
                    Ok(FormulaExpr::FunctionCall { name, args })
                } else {
                    Err(format!("Unrecognized identifier: {}", name))
                }
            }
            _ => Err(format!("Unexpected token in expression: {:?}", tok)),
        }
    }

    fn infix_binding_power(tok: &Token) -> Option<(u8, u8, BinaryOp)> {
        match tok {
            Token::Equal => Some((10, 11, BinaryOp::Eq)),
            Token::NotEqual => Some((10, 11, BinaryOp::NotEq)),
            Token::LessThan => Some((10, 11, BinaryOp::Lt)),
            Token::LessEqual => Some((10, 11, BinaryOp::LtEq)),
            Token::GreaterThan => Some((10, 11, BinaryOp::Gt)),
            Token::GreaterEqual => Some((10, 11, BinaryOp::GtEq)),
            Token::Ampersand => Some((20, 21, BinaryOp::Concat)),
            Token::Plus => Some((30, 31, BinaryOp::Add)),
            Token::Minus => Some((30, 31, BinaryOp::Sub)),
            Token::Star => Some((40, 41, BinaryOp::Mul)),
            Token::Slash => Some((40, 41, BinaryOp::Div)),
            Token::Caret => Some((51, 50, BinaryOp::Power)), // Right-associative
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::token::Lexer;

    #[test]
    fn test_parser_precedence() {
        let mut lexer = Lexer::new("=1 + 2 * 3 ^ 2");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        // Should parse as: 1 + (2 * (3 ^ 2))
        if let FormulaExpr::Binary(BinaryOp::Add, lhs, rhs) = ast {
            assert_eq!(*lhs, FormulaExpr::Literal(CellValue::Number(1.0)));
            if let FormulaExpr::Binary(BinaryOp::Mul, m_lhs, m_rhs) = *rhs {
                assert_eq!(*m_lhs, FormulaExpr::Literal(CellValue::Number(2.0)));
                assert!(matches!(*m_rhs, FormulaExpr::Binary(BinaryOp::Power, _, _)));
            } else {
                panic!("Expected Mul");
            }
        } else {
            panic!("Expected Add");
        }
    }
}
