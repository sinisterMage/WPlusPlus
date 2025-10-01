use crate::ast::{Expr, Node};
use std::mem;
use crate::lexer::{Token, TokenKind};

/// Simple W++ parser that turns text into AST nodes.
/// This can later be replaced with your real parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Main entrypoint: parse an entire program into AST nodes
    pub fn parse_program(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();

        while !self.check(TokenKind::EOF) {
            if let Some(node) = self.parse_stmt() {
                nodes.push(node);
            } else {
                // skip unrecognized token instead of infinite loop
                self.advance();
            }
        }

        nodes
    }
}
impl Parser {
    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn advance(&mut self) -> &TokenKind {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1].kind
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek() == &kind
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.peek() == kind {
                self.advance();
                return true;
            }
        }
        false
    }
    fn matches_symbol(&mut self, sym: &str) -> bool {
    if let TokenKind::Symbol(s) = self.peek() {
        if s == sym {
            self.advance();
            return true;
        }
    }
    false
}

}
impl Parser {
    fn parse_stmt(&mut self) -> Option<Node> {
        match self.peek() {
            TokenKind::Keyword(k) if k == "let" => {
                self.advance();
                self.parse_let()
            }

            TokenKind::Keyword(k) if k == "if" => {
                self.advance();
                Some(self.parse_if())
            }

            _ => {
                // expression statement (like print(...); or a = 5;)
                let expr = self.parse_expr();
                if self.check(TokenKind::Symbol(";".into())) {
                    self.advance();
                }
                Some(Node::Expr(expr))
            }
        }
    }


    fn parse_let(&mut self) -> Option<Node> {
        if let TokenKind::Identifier(name) = self.advance().clone() {
            // expect '='
            if let TokenKind::Symbol(s) = self.advance().clone() {
                if s == "=" {
                    let expr = self.parse_expr();
                    // expect optional ';'
                    if let TokenKind::Symbol(semi) = self.peek() {
                        if semi == ";" {
                            self.advance();
                        }
                    }
                    return Some(Node::Let { name, value: expr });
                }
            }
        }
        None
    }
    fn parse_if(&mut self) -> Node {
        // parse condition (must start with "(")
        self.expect(TokenKind::Symbol("(".into()), "Expected '(' after 'if'");
        let condition = self.parse_expr();
        self.expect(TokenKind::Symbol(")".into()), "Expected ')' after condition");

        // parse body block
        let then_block = self.parse_block();

        // optional else
        let else_block = if self.matches(&[TokenKind::Keyword("else".into())]) {
            if self.check(TokenKind::Keyword("if".into())) {
                // else if chaining (we can support this later)
                let else_if = self.parse_if();
                Some(vec![else_if])
            } else {
                Some(self.parse_block())
            }
        } else {
            None
        };

        Node::Expr(Expr::If {
            cond: Box::new(condition),
            then_branch: then_block,
            else_branch: else_block,
        })
    }
}
impl Parser {
    /// Entry point for expression parsing
    pub fn parse_expr(&mut self) -> Expr {
    self.parse_assignment()
}

fn parse_assignment(&mut self) -> Expr {
    let left = self.parse_equality();

    if self.matches(&[TokenKind::Symbol("=".into())]) {
        let op = if let TokenKind::Symbol(op) = self.tokens[self.pos - 1].kind.clone() {
            op
        } else {
            unreachable!()
        };
        let right = self.parse_assignment(); // allow chaining
        return Expr::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    left
}

}
impl Parser {
    fn parse_equality(&mut self) -> Expr {
        let mut expr = self.parse_comparison();

        while self.matches(&[
            TokenKind::Symbol("==".into()),
            TokenKind::Symbol("!=".into())
        ]) {
            let op = if let TokenKind::Symbol(op) = self.tokens[self.pos - 1].kind.clone() { op } else { unreachable!() };
            let right = self.parse_comparison();
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        expr
    }
}impl Parser {
    fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_term();

        while self.matches(&[
            TokenKind::Symbol("<".into()),
            TokenKind::Symbol(">".into()),
            TokenKind::Symbol("<=".into()),
            TokenKind::Symbol(">=".into())
        ]) {
            let op = if let TokenKind::Symbol(op) = self.tokens[self.pos - 1].kind.clone() { op } else { unreachable!() };
            let right = self.parse_term();
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        expr
    }
}impl Parser {
    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();

        while self.matches(&[
            TokenKind::Symbol("+".into()),
            TokenKind::Symbol("-".into())
        ]) {
            let op = if let TokenKind::Symbol(op) = self.tokens[self.pos - 1].kind.clone() { op } else { unreachable!() };
            let right = self.parse_factor();
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        expr
    }
}impl Parser {
    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_unary();

        while self.matches(&[
            TokenKind::Symbol("*".into()),
            TokenKind::Symbol("/".into())
        ]) {
            let op = if let TokenKind::Symbol(op) = self.tokens[self.pos - 1].kind.clone() { op } else { unreachable!() };
            let right = self.parse_unary();
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        expr
    }
}impl Parser {
    fn parse_unary(&mut self) -> Expr {
        if self.matches(&[
            TokenKind::Symbol("-".into()),
            TokenKind::Symbol("!".into())
        ]) {
            let op = if let TokenKind::Symbol(op) = self.tokens[self.pos - 1].kind.clone() { op } else { unreachable!() };
            let right = self.parse_unary();
            return Expr::BinaryOp {
                left: Box::new(Expr::Literal(0)),
                op,
                right: Box::new(right),
            };
        }

        self.parse_primary()
    }
}impl Parser {
    fn parse_primary(&mut self) -> Expr {
        match self.advance().clone() {
            TokenKind::Number(n) => Expr::Literal(n),
            TokenKind::String(s) => Expr::StringLiteral(s),
            TokenKind::Identifier(name) => {
                // Function call?
                if self.matches(&[TokenKind::Symbol("(".into())]) {
                    let mut args = Vec::new();

                    if !self.check(TokenKind::Symbol(")".into())) {
                        loop {
                            args.push(self.parse_expr());
                            if !self.matches(&[TokenKind::Symbol(",".into())]) {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::Symbol(")".into()), "Expected ')' after function args");
                    Expr::Call { name, args }
                } else {
                    Expr::Variable(name)
                }
            }

            TokenKind::Keyword(k) if k == "true" => Expr::BoolLiteral(true),
            TokenKind::Keyword(k) if k == "false" => Expr::BoolLiteral(false),

            TokenKind::Symbol(sym) if sym == "(".to_string() => {
                let expr = self.parse_expr();
                self.expect(TokenKind::Symbol(")".into()), "Expected ')' after group");
                expr
            }

            _ => panic!("Unexpected token in expression"),
        }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) {
        if self.check(kind.clone()) {
            self.advance();
        } else {
            panic!("{}", msg);
        }
    }
}
impl Parser {
    fn parse_block(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();

        self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start block");

        while !self.check(TokenKind::Symbol("}".into())) && !self.check(TokenKind::EOF) {
            if let Some(stmt) = self.parse_stmt() {
                nodes.push(stmt);
            } else {
                self.advance(); // skip unknown tokens gracefully
            }
        }

        self.expect(TokenKind::Symbol("}".into()), "Expected '}' to close block");
        nodes
    }
}





