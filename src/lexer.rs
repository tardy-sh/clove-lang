use crate::ast::Token;

/// Position in source code for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position { line, column, offset }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Errors that can occur during lexical analysis
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    /// Unexpected character in input
    UnexpectedChar { char: char, position: Position },
    /// Unterminated string literal
    UnterminatedString { position: Position },
    /// Invalid escape sequence in string
    InvalidEscape { char: char, position: Position },
    /// Unexpected EOF (e.g., in middle of string)
    UnexpectedEof { context: String, position: Position },
    /// Bare '=' without '=='
    BareEquals { position: Position },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnexpectedChar { char, position } => {
                write!(f, "Unexpected character '{}' at {}", char, position)
            }
            LexError::UnterminatedString { position } => {
                write!(f, "Unterminated string at {}", position)
            }
            LexError::InvalidEscape { char, position } => {
                write!(f, "Invalid escape sequence '\\{}' at {}", char, position)
            }
            LexError::UnexpectedEof { context, position } => {
                write!(f, "Unexpected end of input {} at {}", context, position)
            }
            LexError::BareEquals { position } => {
                write!(f, "Unexpected '=' at {} (did you mean '==', '!=' or ':='?)", position)
            }
        }
    }
}

impl std::error::Error for LexError {}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.position)
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_string(&mut self, quote: char) -> Result<String, LexError> {
        let start_pos = self.current_position();
        let mut result = String::new();
        self.advance(); // Consume opening quote

        while let Some(ch) = self.current_char() {
            match ch {
                c if c == quote => {
                    self.advance();
                    return Ok(result);
                }
                '\\' => {
                    self.advance(); // Consume backslash
                    let escape_pos = self.current_position();
                    match self.current_char() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('"') => result.push('"'),
                        Some('\'') => result.push('\''),
                        Some('\\') => result.push('\\'),
                        Some(ch) => {
                            return Err(LexError::InvalidEscape {
                                char: ch,
                                position: escape_pos,
                            });
                        }
                        None => {
                            return Err(LexError::UnexpectedEof {
                                context: "after backslash in string".to_string(),
                                position: escape_pos,
                            });
                        }
                    }
                    self.advance();
                }
                _ => {
                    result.push(ch);
                    self.advance();
                }
            }
        }

        Err(LexError::UnterminatedString { position: start_pos })
    }

    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.'
                && !is_float
                && self.peek_char(1).is_some_and(|c| c.is_ascii_digit())
            {
                is_float = true;
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(number.parse::<f64>().expect("Invalid float"))
        } else {
            Token::Integer(number.parse::<i64>().expect("Invalid integer"))
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        match self.current_char() {
            None => Ok(Token::Eof),
            Some('$') => {
                if self
                    .peek_char(1)
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                {
                    self.advance();
                    let name = self.read_identifier();
                    Ok(Token::EnvVar(name))
                } else {
                    self.advance();
                    Ok(Token::Dollar)
                }
            }
            Some('&') => {
                self.advance();
                Ok(Token::Ampersand)
            }
            Some('|') => {
                self.advance();
                Ok(Token::Pipe)
            }
            Some('.') => {
                self.advance();
                Ok(Token::Dot)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('-') => {
                self.advance();
                Ok(Token::Minus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Star)
            }
            Some('/') => {
                self.advance();
                Ok(Token::Slash)
            }
            Some('%') => {
                self.advance();
                Ok(Token::Percent)
            }
            Some('?') => {
                self.advance();
                Ok(Token::Question)
            }
            Some('~') => {
                self.advance();
                Ok(Token::Tilde)
            }
            Some('=') => {
                let pos = self.current_position();
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::EqEq)
                } else {
                    Err(LexError::BareEquals { position: pos })
                }
            }
            Some(':') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::ColonEqual)
                } else {
                    self.advance();
                    Ok(Token::Colon)
                }
            }
            Some('>') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::GtEq)
                } else {
                    self.advance();
                    Ok(Token::Gt)
                }
            }
            Some('<') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::LtEq)
                } else {
                    self.advance();
                    Ok(Token::Lt)
                }
            }
            Some('!') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Token::NotEq)
                } else {
                    self.advance();
                    Ok(Token::Exclamation)
                }
            }
            Some('{') => {
                self.advance();
                Ok(Token::LBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RBrace)
            }
            Some('"') => Ok(Token::String(self.read_string('"')?)),
            Some('\'') => Ok(Token::String(self.read_string('\'')?)),
            Some('@') => {
                self.advance();
                Ok(Token::At)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some('[') => {
                self.advance();
                Ok(Token::LBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::RBracket)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();

                match ident.as_str() {
                    "and" => Ok(Token::And),
                    "or" => Ok(Token::Or),
                    "true" => Ok(Token::Boolean(true)),
                    "false" => Ok(Token::Boolean(false)),
                    "null" => Ok(Token::Null),
                    _ => Ok(Token::Identifier(ident)),
                }
            }
            Some(ch) if ch.is_ascii_digit() => Ok(self.read_number()),
            Some(ch) => {
                let pos = self.current_position();
                Err(LexError::UnexpectedChar { char: ch, position: pos })
            }
        }
    }
}

#[test]
fn test_keywords() {
    let mut lexer = Lexer::new("and or true false null");
    assert_eq!(lexer.next_token().unwrap(), Token::And);
    assert_eq!(lexer.next_token().unwrap(), Token::Or);
    assert_eq!(lexer.next_token().unwrap(), Token::Boolean(true));
    assert_eq!(lexer.next_token().unwrap(), Token::Boolean(false));
    assert_eq!(lexer.next_token().unwrap(), Token::Null);
}

#[test]
fn test_pipe() {
    let mut lexer = Lexer::new("$ | ?($[x] > 5)");
    assert_eq!(lexer.next_token().unwrap(), Token::Dollar);
    assert_eq!(lexer.next_token().unwrap(), Token::Pipe);
    assert_eq!(lexer.next_token().unwrap(), Token::Question);
    assert_eq!(lexer.next_token().unwrap(), Token::LParen);
    assert_eq!(lexer.next_token().unwrap(), Token::Dollar);
    assert_eq!(lexer.next_token().unwrap(), Token::LBracket);
    assert_eq!(lexer.next_token().unwrap(), Token::Identifier("x".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::RBracket);
    assert_eq!(lexer.next_token().unwrap(), Token::Gt);
    assert_eq!(lexer.next_token().unwrap(), Token::Integer(5));
    assert_eq!(lexer.next_token().unwrap(), Token::RParen);
}
