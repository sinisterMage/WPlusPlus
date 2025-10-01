use crate::ast::{Expr, Node};
use std::mem;
use crate::lexer::{Token, TokenKind};
use std::collections::HashMap;


/// Simple W++ parser that turns text into AST nodes.
/// This can later be replaced with your real parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
        pub functions: HashMap<String, Expr>, 
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, functions: HashMap::new(),
 }
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
fn expect_symbol(&mut self, sym: &str) {
    if self.check(TokenKind::Symbol(sym.to_string())) {
        self.advance();
    } else {
        panic!("Expected symbol '{}'", sym);
    }
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

        TokenKind::Keyword(k) if k == "while" => {
            self.advance();
            Some(Node::Expr(self.parse_while()))
        }

        TokenKind::Keyword(k) if k == "for" => {
            self.advance();
            Some(Node::Expr(self.parse_for()))
        }

        TokenKind::Keyword(k) if k == "break" => {
            self.advance();
            if self.check(TokenKind::Symbol(";".into())) { self.advance(); }
            Some(Node::Expr(Expr::Break))
        }

        TokenKind::Keyword(k) if k == "continue" => {
            self.advance();
            if self.check(TokenKind::Symbol(";".into())) { self.advance(); }
            Some(Node::Expr(Expr::Continue))
        }
        TokenKind::Keyword(k) if k == "switch" => {
    self.advance();
    Some(Node::Expr(self.parse_switch()))
}
        TokenKind::Keyword(k) if k == "try" => {
    self.advance();
    Some(Node::Expr(self.parse_try_catch()))
}
TokenKind::Keyword(k) if k == "throw" => {
    self.advance();
    Some(Node::Expr(self.parse_throw()))
}
TokenKind::Keyword(k) if k == "funcy" => {
    self.advance();
    let expr = self.parse_funcy();
    Some(Node::Expr(expr))
}





        _ => {
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
    fn parse_while(&mut self) -> Expr {
    // Expect '('
    self.expect(TokenKind::Symbol("(".into()), "Expected '(' after 'while'");
    let cond = self.parse_expr();
    self.expect(TokenKind::Symbol(")".into()), "Expected ')' after while condition");

    // Expect block start '{'
    if !self.check(TokenKind::Symbol("{".into())) {
        panic!("Expected '{{' to start while block");
    }

    // Parse body
    let body = self.parse_block();

    Expr::While {
        cond: Box::new(cond),
        body,
    }
}
fn parse_for(&mut self) -> Expr {
    self.expect(TokenKind::Symbol("(".into()), "Expected '(' after 'for'");

    // --- Parse initializer ---
    let mut init: Option<Node> = None;

    if !self.check(TokenKind::Symbol(";".into())) {
        if self.check(TokenKind::Keyword("let".into())) {
            // 👇 parse_let() already consumes its trailing ';'
            self.advance(); // consume 'let'
            if let Some(node) = self.parse_let() {
                init = Some(node);
            }
        } else {
            // parse expression initializer
            let expr = self.parse_expr();
            init = Some(Node::Expr(expr));

            // explicitly consume ';' here
            self.expect(TokenKind::Symbol(";".into()), "Expected ';' after for-init expression");
        }
    } else {
        // skip empty initializer
        self.advance();
    }

    // ✅ if we had a `let`, parse_let() already handled its ';'
    // so we skip this check entirely — DO NOT double-expect

    // --- Parse condition ---
    let mut cond: Option<Expr> = None;
    if !self.check(TokenKind::Symbol(";".into())) {
        cond = Some(self.parse_expr());
    }
    self.expect(TokenKind::Symbol(";".into()), "Expected ';' after for-condition");

    // --- Parse post expression ---
let mut post: Option<Expr> = None;
if !self.check(TokenKind::Symbol(")".into())) {
    post = Some(self.parse_expr());
}

// ✅ Instead of self.expect(), do a conditional advance:
if self.check(TokenKind::Symbol(")".into())) {
    self.advance(); // consume ')'
} else {
    panic!("Expected ')' after for-header");
}

// --- Parse body ---
// --- Parse body ---
if !self.check(TokenKind::Symbol("{".into())) {
    panic!("Expected '{{' to start for-body");
}
let body: Vec<Node> = self.parse_block();

Expr::For {
    init: init.map(Box::new),
    cond: cond.map(Box::new),
    post: post.map(Box::new),
    body,
}

}


fn parse_switch(&mut self) -> Expr {
    self.expect(TokenKind::Symbol("(".into()), "Expected '(' after 'switch'");
    let switch_expr = self.parse_expr();
    self.expect(TokenKind::Symbol(")".into()), "Expected ')' after switch expression");

    self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start switch block");

    let mut cases: Vec<(Expr, Vec<Node>)> = Vec::new();
    let mut default: Option<Vec<Node>> = None;

    while !self.check(TokenKind::Symbol("}".into())) && !self.check(TokenKind::EOF) {
        match self.peek() {
            TokenKind::Keyword(k) if k == "case" => {
                self.advance(); // consume 'case'
                let case_expr = self.parse_expr();
                self.expect(TokenKind::Symbol(":".into()), "Expected ':' after case value");
                let body = self.parse_case_body();
                cases.push((case_expr, body));
            }
            TokenKind::Keyword(k) if k == "default" => {
                self.advance();
                self.expect(TokenKind::Symbol(":".into()), "Expected ':' after 'default'");
                default = Some(self.parse_case_body());
            }
            _ => { self.advance(); }
        }
    }

    self.expect(TokenKind::Symbol("}".into()), "Expected '}' to close switch");

    Expr::Switch {
        expr: Box::new(switch_expr),
        cases,
        default,
    }
}

/// helper to parse everything until next `case`, `default`, or `}`
fn parse_case_body(&mut self) -> Vec<Node> {
    let mut nodes = Vec::new();
    while !self.check(TokenKind::Keyword("case".into()))
        && !self.check(TokenKind::Keyword("default".into()))
        && !self.check(TokenKind::Symbol("}".into()))
        && !self.check(TokenKind::EOF)
    {
        if let Some(stmt) = self.parse_stmt() {
            nodes.push(stmt);
        } else {
            self.advance();
        }
    }
    nodes
}
fn parse_try_catch(&mut self) -> Expr {
    let try_block = self.parse_block();

    let mut catch_var = None;
    let mut catch_block = Vec::new();
    let mut finally_block = None;

    // --- Parse optional catch
    if self.matches(&[TokenKind::Keyword("catch".into())]) {
        self.expect(TokenKind::Symbol("(".into()), "Expected '(' after catch");
        if let TokenKind::Identifier(name) = self.advance().clone() {
            catch_var = Some(name);
        }
        self.expect(TokenKind::Symbol(")".into()), "Expected ')' after catch variable");
        catch_block = self.parse_block();
    }

    // --- Parse optional finally
    if self.matches(&[TokenKind::Keyword("finally".into())]) {
        finally_block = Some(self.parse_block());
    }

    Expr::TryCatch {
        try_block,
        catch_var,
        catch_block,
        finally_block,
    }
}

fn parse_throw(&mut self) -> Expr {
    let expr = self.parse_expr();
    if self.check(TokenKind::Symbol(";".into())) {
        self.advance();
    }
    Expr::Throw { expr: Box::new(expr) }
}
fn parse_funcy(&mut self) -> Expr {
    // expect function name
    let name = match self.advance().clone() {
        TokenKind::Identifier(n) => n,
        other => panic!("Expected function name after 'funcy', got {:?}", other),
    };

    // expect '('
    self.expect(TokenKind::Symbol("(".into()), "Expected '(' after function name");

    // parse parameters
    let mut params = Vec::new();
    if !self.check(TokenKind::Symbol(")".into())) {
        loop {
            match self.advance().clone() {
                TokenKind::Identifier(p) => params.push(p),
                other => panic!("Expected parameter name, got {:?}", other),
            }
            if self.check(TokenKind::Symbol(",".into())) {
                self.advance();
                continue;
            } else {
                break;
            }
        }
    }

    self.expect(TokenKind::Symbol(")".into()), "Expected ')' after parameters");

    // expect '{'
    self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start function body");

    // parse body until '}'
    let mut body = Vec::new();
    while !self.check(TokenKind::Symbol("}".into())) && !self.check(TokenKind::EOF) {
        if let Some(stmt) = self.parse_stmt() {
            body.push(stmt);
        } else {
            self.advance();
        }
    }

    self.expect(TokenKind::Symbol("}".into()), "Expected '}' to close function body");

    Expr::Funcy { name, params, body }
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

            unexpected => {
    println!("⚠️ Unexpected token in expression: {:?}", unexpected);
    Expr::Literal(0)
}

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
    self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start block");

    let mut nodes = Vec::new();

    while !self.check(TokenKind::Symbol("}".into())) && !self.check(TokenKind::EOF) {
        if let Some(stmt) = self.parse_stmt() {
            nodes.push(stmt);
        } else {
            self.advance();
        }
    }

    self.expect(TokenKind::Symbol("}".into()), "Expected '}' to end block");
    nodes
}


}





