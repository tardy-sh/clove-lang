use crate::{
    ast::{BinOp, Expr, Query, Statement, Token, UDF},
    lexer::{Lexer, LexError},
};
use std::mem;

/// Errors that can occur during parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Lexer error (with position)
    LexError(LexError),
    /// Unexpected token
    UnexpectedToken { expected: String, got: Token },
    /// Invalid syntax
    InvalidSyntax(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::LexError(e) => write!(f, "{}", e),
            ParseError::UnexpectedToken { expected, got } => {
                write!(f, "Expected {}, got {:?}", expected, got)
            }
            ParseError::InvalidSyntax(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError::LexError(e)
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self, ParseError> {
        let current_token = lexer.next_token()?;
        Ok(Parser {
            lexer,
            current_token,
        })
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.current_token) != std::mem::discriminant(&expected) {
            return Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                got: self.current_token.clone(),
            });
        }
        self.advance()
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(&self.current_token) == std::mem::discriminant(token)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match mem::replace(&mut self.current_token, Token::Eof) {
            // Literals
            Token::Float(n) => {
                self.advance()?;
                Ok(Expr::Float(n))
            }
            Token::Integer(n) => {
                self.advance()?;
                Ok(Expr::Integer(n))
            }
            Token::String(s) => {
                self.advance()?;
                Ok(Expr::String(s))
            }
            Token::Boolean(b) => {
                self.advance()?;
                Ok(Expr::Boolean(b))
            }
            Token::Null => {
                self.advance()?;
                Ok(Expr::Null)
            }

            // References
            Token::Dollar => {
                self.advance()?;
                Ok(Expr::Root)
            }
            Token::EnvVar(name) => {
                self.advance()?;
                Ok(Expr::EnvVar(name))
            }
            Token::At => {
                self.advance()?;

                // Disambiguate '@', '@name', '@1'
                match &self.current_token {
                    // @1, @2 -> Argument reference
                    Token::Integer(n) if *n > 0 => {
                        let arg_num = *n as usize;
                        self.advance()?;
                        Ok(Expr::ArgRef(arg_num))
                    }
                    // @identifier -> scope reference
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance()?;
                        Ok(Expr::ScopeRef(name))
                    }
                    // @ alone -> Lambda parameter
                    _ => Ok(Expr::LambdaParam),
                }
            }

            Token::LParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }

            // Unary minus (for negative numbers/negation)
            Token::Minus => {
                self.advance()?;
                let operand = self.parse_primary()?;
                Ok(Expr::BinaryOp {
                    op: BinOp::Subtract,
                    left: Box::new(Expr::Integer(0)),
                    right: Box::new(operand),
                })
            }

            // These should never appear as primary expressions
            Token::Identifier(name) => Err(ParseError::InvalidSyntax(format!(
                "Unexpected identifier '{}' - identifiers must be part of access expressions (use $[{}] or @[{}])",
                name, name, name
            ))),

            // Object literals
            Token::LBrace => {
                self.advance()?;
                self.parse_object_literal()
            }
            // Array literals
            Token::LBracket => {
                self.advance()?;
                self.parse_array_literal()
            }

            // Others also unexpected
            token => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                got: token,
            }),
        }
    }

    fn parse_object_literal(&mut self) -> Result<Expr, ParseError> {
        let mut pairs = vec![];

        while !self.check(&Token::RBrace) {
            let key = match &self.current_token {
                Token::String(s) => s.clone(),
                Token::Identifier(s) => s.clone(),
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "string or identifier as object key".to_string(),
                        got: self.current_token.clone(),
                    })
                }
            };

            self.advance()?;
            self.expect(Token::Colon)?;

            let value = self.parse_expression()?;
            pairs.push((key, value));

            if !self.check(&Token::RBrace) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::RBrace)?;
        Ok(Expr::Object(pairs))
    }

    fn parse_array_literal(&mut self) -> Result<Expr, ParseError> {
        let mut elements = vec![];

        while !self.check(&Token::RBracket) {
            elements.push(self.parse_expression()?);

            if !self.check(&Token::RBracket) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::RBracket)?;
        Ok(Expr::Array(elements))
    }

    fn parse_access(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::LBracket) {
                self.advance()?;

                if self.check(&Token::Question) {
                    self.advance()?;
                    self.expect(Token::RBracket)?;

                    expr = Expr::ExistenceCheck(Box::new(expr));
                    break;
                } else {
                    let key = self.parse_access_key()?;
                    self.expect(Token::RBracket)?;

                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(key),
                    };
                }
            } else if self.check(&Token::Dot) {
                self.advance()?;

                let name = match &self.current_token {
                    Token::Identifier(n) => n.clone(),
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "identifier after '.'".to_string(),
                            got: self.current_token.clone(),
                        })
                    }
                };

                self.advance()?;

                // Check if this is a method call (identifier followed by '(')
                if self.check(&Token::LParen) {
                    self.advance()?;

                    let mut args = Vec::new();

                    while !self.check(&Token::RParen) {
                        args.push(self.parse_expression()?);

                        if !self.check(&Token::RParen) {
                            self.expect(Token::Comma)?;
                        }
                    }

                    self.expect(Token::RParen)?;

                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: name,
                        args,
                    };
                } else {
                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(Expr::Key(name)),
                    };
                }
            } else if self.check(&Token::Question) {
                // Existence check: $[field]? or @[field]?
                self.advance()?;
                expr = Expr::ExistenceCheck(Box::new(expr));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_access_key(&mut self) -> Result<Expr, ParseError> {
        match &self.current_token {
            Token::Identifier(_) | Token::String(_) => {
                match mem::replace(&mut self.current_token, Token::Eof) {
                    Token::Identifier(name) => {
                        self.advance()?;
                        Ok(Expr::Key(name))
                    }
                    Token::String(name) => {
                        self.advance()?;
                        Ok(Expr::Key(name))
                    }
                    _ => unreachable!(),
                }
            }
            _ => self.parse_expression(),
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_access()?;

        loop {
            let op = match &self.current_token {
                Token::Star => BinOp::Multiply,
                Token::Slash => BinOp::Divide,
                Token::Percent => BinOp::Modulo,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_access()?;

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match &self.current_token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Subtract,
                _ => break,
            };

            self.advance()?;
            let right = self.parse_multiplicative()?;

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;

        if let Some(op) = match &self.current_token {
            Token::EqEq => Some(BinOp::Equal),
            Token::NotEq => Some(BinOp::NotEqual),
            Token::Lt => Some(BinOp::LessThan),
            Token::Gt => Some(BinOp::GreaterThan),
            Token::LtEq => Some(BinOp::LessEqual),
            Token::GtEq => Some(BinOp::GreaterEqual),
            _ => None,
        } {
            self.advance()?;
            let right = self.parse_additive()?;

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;

        while self.check(&Token::And) {
            self.advance()?;
            let right = self.parse_comparison()?;

            left = Expr::BinaryOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;

        while self.check(&Token::Or) {
            self.advance()?;
            let right = self.parse_and()?;

            left = Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_null_coalesce(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_or()?;

        while self.check(&Token::DoubleQuestion) {
            self.advance()?;
            let right = self.parse_or()?;

            left = Expr::BinaryOp {
                op: BinOp::NullCoalesce,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_null_coalesce()
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expression()?;
        self.expect(Token::Eof)?;
        Ok(expr)
    }
}

impl Parser {
    /// Parse a complete query
    pub fn parse_query(&mut self) -> Result<Query, ParseError> {
        let mut udfs = vec![];

        while self.check(&Token::Ampersand) {
            udfs.push(self.parse_udf_definition()?);
        }

        let mut statements = vec![];
        let mut output = None;

        self.expect(Token::Dollar)?;
        while self.check(&Token::Pipe) && output.is_none() {
            self.advance()?;

            if self.check(&Token::Exclamation) {
                output = Some(self.parse_output()?);
                break;
            } else {
                statements.push(self.parse_statement()?);
            }
        }

        self.expect(Token::Eof)?;

        Ok(Query {
            udfs,
            statements,
            output,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match &self.current_token {
            Token::Question => self.parse_filter(),
            Token::Tilde => self.parse_transform(),
            Token::At => self.parse_scope_definition_or_access(),
            Token::Minus => {
                // Peek: if '-' followed by '(' it's a delete statement
                // Otherwise fall through to expression parsing
                self.advance()?;
                if self.check(&Token::LParen) {
                    self.parse_delete()
                } else {
                    // Put back the minus context by parsing as negation expression
                    let operand = self.parse_primary()?;
                    let expr = Expr::BinaryOp {
                        op: BinOp::Subtract,
                        left: Box::new(Expr::Integer(0)),
                        right: Box::new(operand),
                    };
                    // Continue parsing the rest of the expression
                    // (access, multiplicative, additive, etc.)
                    Ok(Statement::Access(expr))
                }
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Access(expr))
            }
        }
    }

    fn parse_delete(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::LParen)?;
        let path_expr = self.parse_access()?;
        self.expect(Token::RParen)?;
        Ok(Statement::Delete(path_expr))
    }

    fn parse_filter(&mut self) -> Result<Statement, ParseError> {
        self.advance()?;
        self.expect(Token::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RParen)?;
        Ok(Statement::Filter(condition))
    }

    fn parse_transform(&mut self) -> Result<Statement, ParseError> {
        self.advance()?;
        self.expect(Token::LParen)?;

        let target = self.parse_access()?;
        self.expect(Token::ColonEqual)?;

        let value = if self.check(&Token::Question) {
            self.advance()?;
            self.expect(Token::LParen)?;
            let condition = self.parse_expression()?;
            self.expect(Token::RParen)?;
            Expr::Filter(Box::new(condition))
        } else {
            self.parse_expression()?
        };

        self.expect(Token::RParen)?;

        Ok(Statement::Transform { target, value })
    }

    fn parse_output(&mut self) -> Result<Expr, ParseError> {
        self.advance()?;
        self.expect(Token::LParen)?;
        let expr = self.parse_expression()?;
        self.expect(Token::RParen)?;
        Ok(expr)
    }

    fn parse_udf_definition(&mut self) -> Result<UDF, ParseError> {
        self.expect(Token::Ampersand)?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "UDF name".to_string(),
                    got: self.current_token.clone(),
                })
            }
        };

        self.advance()?;
        self.expect(Token::Colon)?;

        let arity = match &self.current_token {
            Token::Integer(n) if *n >= 0 => *n as usize,
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "non-negative integer for UDF arity".to_string(),
                    got: self.current_token.clone(),
                })
            }
        };
        self.advance()?;

        self.expect(Token::ColonEqual)?;

        let body = self.parse_statement()?;

        Ok(UDF { name, arity, body })
    }

    fn parse_scope_definition_or_access(&mut self) -> Result<Statement, ParseError> {
        self.advance()?;

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier after '@'".to_string(),
                    got: self.current_token.clone(),
                })
            }
        };

        self.advance()?;

        if self.check(&Token::ColonEqual) {
            self.advance()?;
            let path = self.parse_expression()?;
            Ok(Statement::ScopeDefinition { name, path })
        } else {
            let mut expr = Expr::ScopeRef(name);

            while self.check(&Token::LBracket) || self.check(&Token::Dot) {
                if self.check(&Token::LBracket) {
                    self.advance()?;
                    let key = self.parse_access_key()?;
                    self.expect(Token::RBracket)?;

                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(key),
                    };
                } else if self.check(&Token::Dot) {
                    self.advance()?;

                    let field_name = match &self.current_token {
                        Token::Identifier(n) => n.clone(),
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "identifier after '.'".to_string(),
                                got: self.current_token.clone(),
                            })
                        }
                    };
                    self.advance()?;

                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(Expr::Key(field_name)),
                    };
                }
            }
            Ok(Statement::Access(expr))
        }
    }
}
