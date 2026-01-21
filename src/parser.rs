use crate::{
    ast::{BinOp, Expr, Query, Statement, Token, UDF},
    lexer::Lexer,
};
use std::mem;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn expect(&mut self, expected: Token) {
        if std::mem::discriminant(&self.current_token) != std::mem::discriminant(&expected) {
            panic!("Expected {:?}, got {:?}", expected, self.current_token);
        }
        self.advance();
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(&self.current_token) == std::mem::discriminant(token)
    }

    /// Parse primary expressions (atoms): literal values, '$', '@', '(', ')'
    /// Q: should literal objects not be a part of this?
    fn parse_primary(&mut self) -> Expr {
        match mem::replace(&mut self.current_token, Token::Eof) {
            // Literals
            Token::Float(n) => {
                self.advance();
                Expr::Float(n)
            }
            Token::Integer(n) => {
                self.advance();
                Expr::Integer(n)
            }
            Token::String(s) => {
                self.advance();
                Expr::String(s)
            }
            Token::Boolean(b) => {
                self.advance();
                Expr::Boolean(b)
            }
            Token::Null => {
                self.advance();
                Expr::Null
            }

            // References
            Token::Dollar => {
                self.advance();
                Expr::Root
            }
            Token::EnvVar(name) => {
                self.advance();
                Expr::EnvVar(name)
            }
            Token::At => {
                self.advance();

                // Disambiguate '@', '@name', '@1'
                match &self.current_token {
                    // @1, @2 -> Argument reference
                    Token::Integer(n) if *n > 0 => {
                        let arg_num = *n as usize;
                        self.advance();
                        Expr::ArgRef(arg_num)
                    }
                    // @identifier -> scope reference
                    Token::Identifier(name) => {
                        let name = name.clone();
                        self.advance();
                        Expr::ScopeRef(name)
                    }
                    // @ alone -> Lambda parameter
                    _ => Expr::LambdaParam,
                }
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(Token::RParen);
                expr
            }

            // Unary minuus (for negative numbers/negation)
            Token::Minus => {
                self.advance();
                let operand = self.parse_primary(); // Right-associative
                // Represent as 0 - operandd
                Expr::BinaryOp {
                    op: BinOp::Subtract,
                    left: Box::new(Expr::Integer(0)),
                    right: Box::new(operand),
                }
            }

            // These should never appear as primary expressions
            Token::Identifier(_) => {
                panic!(
                    "Unexpected use of identifiers - identifiers must be a part of access expressions"
                )
            }
            // Object literals
            Token::LBrace => {
                self.advance();
                self.parse_object_literal()
            }
            // Array literals
            Token::LBracket => {
                self.advance();
                self.parse_array_literal()
            }

            // Others also unexpected, for now handled together
            token => panic!("Unexpected token in primary expression: {:?}", token),
        }
    }

    fn parse_object_literal(&mut self) -> Expr {
        let mut pairs = vec![];

        while !self.check(&Token::RBrace) {
            let key = match &self.current_token {
                Token::String(s) => s.clone(),
                Token::Identifier(s) => s.clone(),
                _ => panic!("Expected string or identifier as object key"),
            };

            self.advance();

            self.expect(Token::Colon);

            let value = self.parse_expression();
            pairs.push((key, value));

            if !self.check(&Token::RBrace) {
                self.expect(Token::Comma);
            }
        }

        self.expect(Token::RBrace);
        Expr::Object(pairs)
    }

    fn parse_array_literal(&mut self) -> Expr {
        let mut elements = vec![];

        while !self.check(&Token::RBracket) {
            elements.push(self.parse_expression());

            if !self.check(&Token::RBracket) {
                self.expect(Token::Comma);
            }
        }

        self.expect(Token::RBracket);
        Expr::Array(elements)
    }

    /// Parse access expressions
    fn parse_access(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        loop {
            if self.check(&Token::LBracket) {
                self.advance(); // Consume '['

                if self.check(&Token::Question) {
                    self.advance(); // consume '?'
                    self.expect(Token::RBracket); // consume ']'

                    expr = Expr::ExistenceCheck(Box::new(expr));
                    break;
                } else {
                    let key = self.parse_access_key();

                    self.expect(Token::RBracket);

                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(key),
                    };
                }
            } else if self.check(&Token::Dot) {
                self.advance(); // consume '.'

                // After dot, we expect an identifier
                let name = match &self.current_token {
                    Token::Identifier(n) => n.clone(),
                    _ => panic!(
                        "Expected identifier after '.', got {:?}",
                        self.current_token
                    ),
                };

                self.advance();

                expr = Expr::Access {
                    object: Box::new(expr),
                    key: Box::new(Expr::Key(name)),
                };
            } else {
                break;
            }
        }
        expr
    }

    fn parse_access_key(&mut self) -> Expr {
        // Inside brackets, we can have:
        // 1. Identifier -> Key (simple field name)
        // 2. String -> Key (field name with special chars)
        // 3. Number -> Number (array index)
        // 4. ? -> Existence check
        // 5. Any expression -> computed key

        match &self.current_token {
            Token::Identifier(_) | Token::String(_) => {
                match mem::replace(&mut self.current_token, Token::Eof) {
                    Token::Identifier(name) => {
                        self.advance();
                        Expr::Key(name)
                    }
                    Token::String(name) => {
                        self.advance();
                        Expr::Key(name)
                    }
                    _ => unreachable!(),
                }
            }
            _ => self.parse_expression(),
        }
    }

    fn parse_multiplicative(&mut self) -> Expr {
        // Q: do we also want to add ** and //? Maybe also -- and ++
        let mut left = self.parse_access();

        loop {
            let op = match &self.current_token {
                Token::Star => BinOp::Multiply,
                Token::Slash => BinOp::Divide,
                Token::Percent => BinOp::Modulo,
                _ => break,
            };

            self.advance();
            let right = self.parse_access();

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();

        loop {
            let op = match &self.current_token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplicative(); // We start from this

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_additive();

        if let Some(op) = match &self.current_token {
            Token::EqEq => Some(BinOp::Equal),
            Token::NotEq => Some(BinOp::NotEqual),
            Token::Lt => Some(BinOp::LessThan),
            Token::Gt => Some(BinOp::GreaterThan),
            Token::LtEq => Some(BinOp::LessEqual),
            Token::GtEq => Some(BinOp::GreaterEqual),
            _ => None,
        } {
            self.advance();
            let right = self.parse_additive();

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_comparison();

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_comparison();

            left = Expr::BinaryOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and();

            left = Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        left
    }

    pub fn parse_expression(&mut self) -> Expr {
        self.parse_or()
    }

    pub fn parse(&mut self) -> Expr {
        let expr = self.parse_expression();
        self.expect(Token::Eof);
        expr
    }
}

impl Parser {
    /// Parse a complete query
    pub fn parse_query(&mut self) -> Query {
        let mut udfs = vec![];

        while self.check(&Token::Ampersand) {
            udfs.push(self.parse_udf_definition());
        }

        let mut statements = vec![];
        let mut output = None;

        // if !self.check(&Token::Dollar) {
        //     if self.check(&Token::Exclamation) {
        //         output = Some(self.parse_output());
        //     } else {
        //         statements.push(self.parse_statement());
        //     }
        // }

        self.expect(Token::Dollar);
        while self.check(&Token::Pipe) && output.is_none() {
            self.advance();

            if self.check(&Token::Exclamation) {
                output = Some(self.parse_output());
                break;
            } else {
                statements.push(self.parse_statement());
            }
        }


        self.expect(Token::Eof);

        Query {
            udfs,
            statements,
            output,
        }
    }

    fn parse_statement(&mut self) -> Statement {
        match &self.current_token {
            Token::Question => self.parse_filter(),
            Token::Tilde => self.parse_transform(),
            Token::At => self.parse_scope_definition_or_access(),
            _ => {
                let expr = self.parse_expression();
                Statement::Access(expr)
            }
        }
    }

    fn parse_filter(&mut self) -> Statement {
        self.advance(); // consume ?
        self.expect(Token::LParen);
        let condition = self.parse_expression();
        self.expect(Token::RParen);
        Statement::Filter(condition)
    }

    fn parse_transform(&mut self) -> Statement {
        self.advance(); // consume ~
        self.expect(Token::LParen);

        // Restriction: target must be access path (field, array index, or scope ref)
        let target = self.parse_access();

        self.expect(Token::ColonEqual);

        // Value can be any expression
        let value = if self.check(&Token::Question) {
            self.advance();
            self.expect(Token::LParen);
            let condition = self.parse_expression();
            self.expect(Token::RParen);
            Expr::Filter(Box::new(condition))
        } else {
            self.parse_expression()
        };

        self.expect(Token::RParen);

        Statement::Transform { target, value }
    }

    fn parse_output(&mut self) -> Expr {
        self.advance(); // Consume !
        self.expect(Token::LParen);
        let expr = self.parse_expression();
        self.expect(Token::RParen);
        expr
    }

    fn parse_udf_definition(&mut self) -> UDF {
        self.expect(Token::Ampersand);

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => panic!("Expected UDF name after `&`, got {:?}", self.current_token),
        };

        self.advance();

        self.expect(Token::Colon);

        let arity = match &self.current_token {
            Token::Integer(n) if *n >= 0 => *n as usize,
            _ => panic!(
                "Expected non-negative integer for UDF arity, got {:?}",
                self.current_token
            ),
        };
        self.advance();

        self.expect(Token::ColonEqual);

        let body = self.parse_statement();

        UDF { name, arity, body }
    }

    fn parse_scope_definition_or_access(&mut self) -> Statement {
        self.advance();

        let name = match &self.current_token {
            Token::Identifier(n) => n.clone(),
            _ => panic!(
                "Expected identifier after `@`, got {:?}",
                self.current_token
            ),
        };

        self.advance();

        if self.check(&Token::ColonEqual) {
            self.advance();
            let path = self.parse_expression();
            Statement::ScopeDefinition { name, path }
        } else {
            let mut expr = Expr::ScopeRef(name);

            while self.check(&Token::LBracket) || self.check(&Token::Dot) {
                if self.check(&Token::LBracket) {
                    self.advance();
                    let key = self.parse_access_key();
                    self.expect(Token::RBracket);

                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(key),
                    };
                } else if self.check(&Token::Dot) {
                    self.advance();

                    let field_name = match &self.current_token {
                        Token::Identifier(n) => n.clone(),
                        _ => panic!(
                            "Expected identifier after '.', got {:?}",
                            self.current_token
                        ),
                    };
                    self.advance();

                    expr = Expr::Access {
                        object: Box::new(expr),
                        key: Box::new(Expr::Key(field_name)),
                    };
                }
            }
            Statement::Access(expr)
        }
    }
}
