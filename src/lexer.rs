use crate::ast::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }
    fn advance(&mut self) {
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

    fn read_string(&mut self, quote: char) -> String {
        let mut result = String::new();
        self.advance(); // ✓ Consume opening quote

        while let Some(ch) = self.current_char() {
            match ch {
                c if c == quote => {
                    self.advance();
                    return result;
                }
                '\\' => {
                    self.advance(); // Consume backslash
                    match self.current_char() {
                        Some('n') => result.push('\n'),
                        Some('t') => result.push('\t'),
                        Some('r') => result.push('\r'),
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some(ch) => panic!("Invalid escape sequence: \\{}", ch),
                        None => panic!("Unterminated string: unexpected EOF after backslash"),
                    }
                    self.advance();
                }
                _ => {
                    result.push(ch);
                    self.advance();
                }
            }
        }

        panic!("Unterminated string: missing closing quote");
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

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char() {
            None => Token::Eof,
            Some('$') => {
                if self
                    .peek_char(1)
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                {
                    self.advance();
                    let name = self.read_identifier();
                    Token::EnvVar(name)
                } else {
                    self.advance();
                    Token::Dollar
                }
            }
            Some('&') => {
                self.advance();
                Token::Ampersand
            }
            Some('|') => {
                self.advance();
                Token::Pipe
            }
            Some('.') => {
                self.advance();
                Token::Dot
            }
            Some(',') => {
                self.advance();
                Token::Comma
            }
            Some('+') => {
                self.advance();
                Token::Plus
            }
            Some('-') => {
                self.advance();
                Token::Minus
            }
            Some('*') => {
                self.advance();
                Token::Star
            }
            Some('/') => {
                self.advance();
                Token::Slash
            }
            Some('%') => {
                self.advance();
                Token::Percent
            }
            Some('?') => {
                self.advance();
                Token::Question
            }
            Some('~') => {
                self.advance();
                Token::Tilde
            }
            Some('=') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Token::EqEq
                } else {
                    panic!(
                        "Panic at character '=', position {}:\nUnexpected '=' (did you mean '==', '!=' or ':='?)",
                        self.position,
                    )
                }
            }
            Some(':') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Token::ColonEqual
                } else {
                    self.advance();
                    Token::Colon
                }
            }
            Some('>') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Token::GtEq
                } else {
                    self.advance();
                    Token::Gt
                }
            }
            Some('<') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Token::LtEq
                } else {
                    self.advance();
                    Token::Lt
                }
            }
            Some('!') => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    Token::NotEq
                } else {
                    self.advance();
                    Token::Exclamation
                }
            }
            Some('{') => {
                self.advance();
                Token::LBrace
            }
            Some('}') => {
                self.advance();
                Token::RBrace
            }
            Some('"') => Token::String(self.read_string('"')),
            Some('\'') => Token::String(self.read_string('\'')),
            Some('@') => {
                self.advance();
                Token::At
            }
            Some('(') => {
                self.advance();
                Token::LParen
            }
            Some(')') => {
                self.advance();
                Token::RParen
            }
            Some('[') => {
                self.advance();
                Token::LBracket
            }
            Some(']') => {
                self.advance();
                Token::RBracket
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();

                match ident.as_str() {
                    "and" => Token::And,
                    "or" => Token::Or,
                    "true" => Token::Boolean(true),
                    "false" => Token::Boolean(false),
                    "null" => Token::Null,
                    _ => Token::Identifier(ident),
                }
            }
            Some(ch) if ch.is_ascii_digit() => self.read_number(),
            Some(ch) => panic!(
                "Unexpected character '{}' at position {}",
                ch, self.position
            ),
        }
    }
}

#[test]
fn test_keywords() {
    let mut lexer = Lexer::new("and or true false null");
    assert_eq!(lexer.next_token(), Token::And);
    assert_eq!(lexer.next_token(), Token::Or);
    assert_eq!(lexer.next_token(), Token::Boolean(true));
    assert_eq!(lexer.next_token(), Token::Boolean(false));
    assert_eq!(lexer.next_token(), Token::Null);
}

#[test]
fn test_pipe() {
    let mut lexer = Lexer::new("$ | ?($[x] > 5)");
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::Pipe);
    assert_eq!(lexer.next_token(), Token::Question);
    assert_eq!(lexer.next_token(), Token::LParen);
    assert_eq!(lexer.next_token(), Token::Dollar);
    assert_eq!(lexer.next_token(), Token::LBracket);
    assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
    assert_eq!(lexer.next_token(), Token::RBracket);
    assert_eq!(lexer.next_token(), Token::Gt);
    assert_eq!(lexer.next_token(), Token::Integer(5));
    assert_eq!(lexer.next_token(), Token::RParen);
}
