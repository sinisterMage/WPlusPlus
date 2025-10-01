use std::iter::Peekable;
use std::str::Chars;

/// Kinds of tokens in W++
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(String),     // e.g. let, if, else, while, for, break, continue
    Identifier(String),  // e.g. variable or function name
    Number(i32),         // e.g. 123
    String(String),      // e.g. "hello"
    Symbol(String),      // e.g. { } ( ) ; , = + - * / == != <= >= < >
    EOF,                 // End of file
}

/// A token with line/column metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

/// The W++ lexer (tokenizer)
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from a source string
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source.chars().peekable(),
            line: 1,
            col: 0,
        }
    }

    /// Tokenize the entire source into a vector of tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(&ch) = self.input.peek() {
            match ch {
                // --- Whitespace ---
                c if c.is_whitespace() => {
                    if c == '\n' {
                        self.line += 1;
                        self.col = 0;
                    } else {
                        self.col += 1;
                    }
                    self.input.next();
                }

                // --- Identifiers & keywords ---
                c if c.is_alphabetic() || c == '_' => {
                    let ident = self.consume_identifier();
                    let kind = match ident.as_str() {
                        "let" | "if" | "else" | "while" | "for"
                        | "break" | "continue" | "true" | "false" => {
                            TokenKind::Keyword(ident)
                        }
                        _ => TokenKind::Identifier(ident),
                    };
                    tokens.push(Token {
                        kind,
                        line: self.line,
                        col: self.col,
                    });
                }

                // --- Numbers ---
                c if c.is_ascii_digit() => {
                    let num_str = self.consume_number();
                    let value = num_str.parse::<i32>().unwrap_or_else(|_| {
                        panic!("Invalid number literal on line {}", self.line)
                    });
                    tokens.push(Token {
                        kind: TokenKind::Number(value),
                        line: self.line,
                        col: self.col,
                    });
                }

                // --- Strings ---
                '"' => {
                    self.input.next(); // skip the opening quote
                    let s = self.consume_string();
                    tokens.push(Token {
                        kind: TokenKind::String(s),
                        line: self.line,
                        col: self.col,
                    });
                }

                // --- Operators and symbols ---
                _ => {
                    let sym = self.consume_symbol();
                    tokens.push(Token {
                        kind: TokenKind::Symbol(sym),
                        line: self.line,
                        col: self.col,
                    });
                }
            }
        }

        tokens.push(Token {
            kind: TokenKind::EOF,
            line: self.line,
            col: self.col,
        });

        tokens
    }

    // ===========================
    // === Helper subroutines ===
    // ===========================

    fn consume_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.input.next();
                self.col += 1;
            } else {
                break;
            }
        }
        ident
    }

    fn consume_number(&mut self) -> String {
        let mut number = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_ascii_digit() {
                number.push(c);
                self.input.next();
                self.col += 1;
            } else {
                break;
            }
        }
        number
    }

    fn consume_string(&mut self) -> String {
        let mut result = String::new();
        while let Some(c) = self.input.next() {
            self.col += 1;
            match c {
                '"' => break, // closing quote
                '\\' => {
                    if let Some(next) = self.input.next() {
                        self.col += 1;
                        match next {
                            'n' => result.push('\n'),
                            't' => result.push('\t'),
                            '"' => result.push('"'),
                            '\\' => result.push('\\'),
                            _ => result.push(next),
                        }
                    }
                }
                _ => result.push(c),
            }
        }
        result
    }

    fn consume_symbol(&mut self) -> String {
        let ch = self.input.next().unwrap();
        self.col += 1;

        // Handle two-character operators
        if let Some(&next) = self.input.peek() {
            let pair = format!("{}{}", ch, next);
            if ["==", "!=", "<=", ">="].contains(&pair.as_str()) {
                self.input.next();
                self.col += 1;
                return pair;
            }
        }

        // Single-character symbols
        ch.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let mut lexer = Lexer::new(r#"let x = 10; while x < 20 { print("hi"); }"#);
        let tokens = lexer.tokenize();

        for t in &tokens {
            println!("{:?}", t);
        }

        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Keyword(ref s) if s == "let")));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Identifier(ref s) if s == "x")));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Number(10))));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Symbol(ref s) if s == "{")));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Symbol(ref s) if s == "}")));
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::EOF));
    }
}
