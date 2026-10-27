use crate::cell::CellCoordinate;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    StringLiteral(String),
    Boolean(bool),
    CellRef(CellCoordinate),
    CellRangeRef { start: CellCoordinate, end: CellCoordinate },
    SpillRef(CellCoordinate),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Ampersand,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    OpenParen,
    CloseParen,
    Comma,
    Colon,
    Hash,
}

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let clean = input.trim_start_matches('=');
        Self {
            chars: clean.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
                continue;
            }

            match ch {
                '+' => { self.chars.next(); tokens.push(Token::Plus); }
                '-' => { self.chars.next(); tokens.push(Token::Minus); }
                '*' => { self.chars.next(); tokens.push(Token::Star); }
                '/' => { self.chars.next(); tokens.push(Token::Slash); }
                '^' => { self.chars.next(); tokens.push(Token::Caret); }
                '&' => { self.chars.next(); tokens.push(Token::Ampersand); }
                '(' => { self.chars.next(); tokens.push(Token::OpenParen); }
                ')' => { self.chars.next(); tokens.push(Token::CloseParen); }
                ',' => { self.chars.next(); tokens.push(Token::Comma); }
                ':' => { self.chars.next(); tokens.push(Token::Colon); }
                '#' => { self.chars.next(); tokens.push(Token::Hash); }
                '=' => { self.chars.next(); tokens.push(Token::Equal); }
                '<' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::LessEqual);
                    } else if let Some(&'>') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::NotEqual);
                    } else {
                        tokens.push(Token::LessThan);
                    }
                }
                '>' => {
                    self.chars.next();
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::GreaterThan);
                    }
                }
                '"' => {
                    self.chars.next();
                    let mut s = String::new();
                    while let Some(&c) = self.chars.peek() {
                        self.chars.next();
                        if c == '"' {
                            if let Some(&'"') = self.chars.peek() {
                                self.chars.next();
                                s.push('"');
                            } else {
                                break;
                            }
                        } else {
                            s.push(c);
                        }
                    }
                    tokens.push(Token::StringLiteral(s));
                }
                '0'..='9' | '.' => {
                    let mut num_str = String::new();
                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            num_str.push(c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    let num = num_str.parse::<f64>().map_err(|e| e.to_string())?;
                    tokens.push(Token::Number(num));
                }
                'a'..='z' | 'A'..='Z' | '_' | '$' => {
                    let mut id = String::new();
                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '.' {
                            id.push(c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }

                    // Check if it's a cell reference e.g. A1, $B$10
                    if let Some(coord) = CellCoordinate::parse(&id) {
                        tokens.push(Token::CellRef(coord));
                    } else if id.eq_ignore_ascii_case("TRUE") {
                        tokens.push(Token::Boolean(true));
                    } else if id.eq_ignore_ascii_case("FALSE") {
                        tokens.push(Token::Boolean(false));
                    } else {
                        tokens.push(Token::Ident(id.to_ascii_uppercase()));
                    }
                }
                _ => {
                    return Err(format!("Unexpected character: {}", ch));
                }
            }
        }

        // Post-process range tokens: CellRef Colon CellRef -> CellRangeRef
        let mut coalesced = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            if i + 2 < tokens.len() {
                if let (Token::CellRef(c1), Token::Colon, Token::CellRef(c2)) = (&tokens[i], &tokens[i + 1], &tokens[i + 2]) {
                    coalesced.push(Token::CellRangeRef { start: *c1, end: *c2 });
                    i += 3;
                    continue;
                }
            }
            if i + 1 < tokens.len() {
                if let (Token::CellRef(c), Token::Hash) = (&tokens[i], &tokens[i + 1]) {
                    coalesced.push(Token::SpillRef(*c));
                    i += 2;
                    continue;
                }
            }
            coalesced.push(tokens[i].clone());
            i += 1;
        }

        Ok(coalesced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let mut lexer = Lexer::new("=SUM(A1:B10, 42.5, \"Total\")");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Ident("SUM".into()));
        assert_eq!(tokens[1], Token::OpenParen);
        assert_eq!(
            tokens[2],
            Token::CellRangeRef {
                start: CellCoordinate::new(0, 0),
                end: CellCoordinate::new(9, 1)
            }
        );
        assert_eq!(tokens[3], Token::Comma);
        assert_eq!(tokens[4], Token::Number(42.5));
    }
}
