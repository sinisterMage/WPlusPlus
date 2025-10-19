use std::iter::Peekable;
use std::str::Chars;
use unicode_normalization::UnicodeNormalization;


/// Kinds of tokens in W++
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword(String),     // e.g. let, if, else, while, for, break, continue
    Identifier(String),  // e.g. variable or function name
    Number {
    raw: String,
    ty: String, // e.g. "i32", "u64", "f64"
},

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
                c if is_identifier_start(c) => {
                     let ident = self.consume_identifier();
                    let kind = match ident.as_str() {
    "let" | "if" | "else" | "while" | "for"
    | "break" | "continue" | "true" | "false"
    | "switch" | "case" | "default" | "try" | "catch" | "throw" | "finally" | "funcy" | "return" | "async" | "await" | "const" | "func" | "entity" | "alters" | "me" | "new" | "import" | "export" | "from"=> {
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
    let token = self.read_typed_number();
    tokens.push(Token {
        kind: token,
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
                // --- Operators and symbols ---
_ => {
    if ch == '/' {
        // Peek ahead for comment start
        let mut iter = self.input.clone();
        iter.next(); // skip '/'
        if let Some('/') = iter.next() {
            // ✅ It's a comment — consume the whole line
            self.input.next(); // skip first '/'
            self.input.next(); // skip second '/'
            self.col += 2;
            while let Some(&c) = self.input.peek() {
                if c == '\n' {
                    break;
                }
                self.input.next();
                self.col += 1;
            }
            continue; // 🧠 skip comment entirely
        }
    }

    // Otherwise, normal symbol or operator
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
        if is_identifier_continue(c) {
            ident.push(c);
            self.input.next();
            self.col += 1;
        } else {
            break;
        }
    }

    // Normalize to NFC to ensure consistent representation
    ident.nfc().collect::<String>()
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
    fn read_typed_number(&mut self) -> TokenKind {
    let mut num_str = String::new();
    let mut is_float = false;

    // 1️⃣ Gather digits and optional decimal
    while let Some(&c) = self.input.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            self.input.next();
            self.col += 1;
        } else if c == '.' && !is_float {
            is_float = true;
            num_str.push(c);
            self.input.next();
            self.col += 1;
        } else {
            break;
        }
    }

    // 2️⃣ Gather optional type suffix (like i32, u64, f64, etc.)
    let mut type_str = String::new();
    if let Some(&c) = self.input.peek() {
        if c == 'i' || c == 'u' || c == 'f' {
            while let Some(&next) = self.input.peek() {
                if next.is_ascii_alphanumeric() {
                    type_str.push(next);
                    self.input.next();
                    self.col += 1;
                } else {
                    break;
                }
            }
        }
    }

    // 3️⃣ Default types
    if type_str.is_empty() {
        type_str = if is_float { "f64".to_string() } else { "i32".to_string() };
    }

    TokenKind::Number {
        raw: num_str,
        ty: type_str,
    }
}

}
// =======================================
// === Identifier helper functions =======
// =======================================

fn is_identifier_start(ch: char) -> bool {
    // Allow underscore, letters, and most emoji / symbol codepoints
    ch == '_'
        || ch.is_alphabetic()
        || (ch >= '\u{1F300}' && ch <= '\u{1FAFF}') // emoji & pictographs
        || (ch >= '\u{2600}' && ch <= '\u{27BF}')   // misc symbols
        || (ch >= '\u{1F900}' && ch <= '\u{1F9FF}') // supplemental symbols
}

fn is_identifier_continue(ch: char) -> bool {
    // Same as start + digits + joiners
    is_identifier_start(ch)
        || ch.is_alphanumeric()
        || ch == '\u{200C}' // Zero-width non-joiner
        || ch == '\u{200D}' // Zero-width joiner
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
        assert!(tokens.iter().any(|t|
    matches!(t.kind, TokenKind::Number { ref raw, ref ty }
        if raw == "10" && ty == "i32"
    )
));

        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Symbol(ref s) if s == "{")));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Symbol(ref s) if s == "}")));
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::EOF));
    }
    #[test]
fn test_utf8_identifiers() {
    let mut lexer = Lexer::new(r#"let 🦥 = 1; let 変数 = 2; let привет = 3;"#);
    let tokens = lexer.tokenize();

    let idents: Vec<_> = tokens.iter().filter_map(|t| {
        if let TokenKind::Identifier(s) = &t.kind { Some(s.clone()) } else { None }
    }).collect();

    assert_eq!(idents, vec!["🦥", "変数", "привет"]);
}

}
