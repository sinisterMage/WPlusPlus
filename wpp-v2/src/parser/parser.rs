use crate::ast::{node::{EntityMember, EntityNode}, Expr, Node};
use std::mem;
use crate::lexer::{Token, TokenKind};
use std::collections::HashMap;
use crate::lexer::Lexer;



/// Simple W++ parser that turns text into AST nodes.
/// This can later be replaced with your real parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
        pub functions: HashMap<String, Expr>, 
}
pub fn parse(source: &str) -> Result<Vec<crate::ast::Node>, String> {
    // 1️⃣ Tokenize the input source
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // 2️⃣ Parse the token stream
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    Ok(ast)
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
    fn peek_type_name(&self) -> bool {
    match self.peek() {
        TokenKind::Identifier(name)
            if ["i32", "u64", "i8", "i1", "f64"].contains(&name.as_str()) => true,
        _ => false,
    }
}

    fn parse_stmt(&mut self) -> Option<Node> {
    match self.peek() {
        TokenKind::Keyword(k) if k == "let" || k == "const" => {
            let is_const = k == "const"; // ✅ determine constness
            self.advance();
            self.parse_let(is_const)
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
TokenKind::Keyword(k) if k == "async" => {
    self.advance(); // consume 'async'
    if self.check(TokenKind::Keyword("funcy".into())) {
        self.advance(); // consume 'funcy'
        let expr = self.parse_funcy(true);
        Some(Node::Expr(expr))
    } else {
        panic!("Expected 'funcy' after 'async'");
    }
}

TokenKind::Keyword(k) if k == "func" || k == "funcy" => {
    self.advance();
    let expr = self.parse_funcy(false);
    Some(Node::Expr(expr))
}




TokenKind::Keyword(k) if k == "return" => {
    self.advance();

    // Optional return expression
    let expr = if !self.check(TokenKind::Symbol(";".into()))
        && !self.check(TokenKind::Symbol("}".into()))
    {
        Some(Box::new(self.parse_expr()))
    } else {
        None
    };

    // Optional semicolon
    if self.check(TokenKind::Symbol(";".into())) {
        self.advance();
    }

    Some(Node::Expr(Expr::Return(expr)))
}
TokenKind::Keyword(k) if k == "entity" => {
    self.parse_entity()
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
pub fn parse_entity(&mut self) -> Option<Node> {
    self.expect(TokenKind::Keyword("entity".into()), "Expected 'entity' keyword");

    // --- Parse entity name ---
    let name = match self.advance().clone() {
        TokenKind::Identifier(n) => n,
        other => panic!("Expected entity name, got {:?}", other),
    };

    // --- Optional inheritance: `entity Dog alters Animal`
    let base = if self.check(TokenKind::Keyword("alters".into())) {
        self.advance();
        match self.advance().clone() {
            TokenKind::Identifier(b) => Some(b),
            other => panic!("Expected base entity name after 'alters', got {:?}", other),
        }
    } else {
        None
    };

    self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start entity block");

    // --- Parse members ---
    let mut members = Vec::new();

    while !self.check(TokenKind::Symbol("}".into())) && !self.check(TokenKind::EOF) {
        match self.peek() {
            // --- Field like: `age = 5;`
            TokenKind::Identifier(_) => {
                let field_name = match self.advance().clone() {
                    TokenKind::Identifier(id) => id,
                    other => panic!("Expected field name, got {:?}", other),
                };

                // Allow both `=` and `:` syntax
                if self.check(TokenKind::Symbol("=".into())) || self.check(TokenKind::Symbol(":".into())) {
                    self.advance();
                } else {
                    panic!("Expected '=' or ':' in field declaration");
                }

                let value = self.parse_expr();
                members.push(EntityMember::Field { name: field_name, value });

                // Optional semicolon or newline
                if self.check(TokenKind::Symbol(";".into())) {
                    self.advance();
                }
            }

            // --- Method like: `func bark() => print("woof");`
            TokenKind::Keyword(k) if k == "func" || k == "funcy" => {
                self.advance();
                let func_expr = self.parse_funcy(false);
                let func_name = match &func_expr {
                    Expr::Funcy { name, .. } => name.clone(),
                    _ => "anonymous".to_string(),
                };
                members.push(EntityMember::Method { name: func_name, func: func_expr });
            }

            // --- Async methods: `async funcy bark() { ... }`
            TokenKind::Keyword(k) if k == "async" => {
                self.advance();
                self.expect(TokenKind::Keyword("funcy".into()), "Expected 'funcy' after 'async'");
                let func_expr = self.parse_funcy(true);
                let func_name = match &func_expr {
                    Expr::Funcy { name, .. } => name.clone(),
                    _ => "anonymous".to_string(),
                };
                members.push(EntityMember::Method { name: func_name, func: func_expr });
            }

            // --- Ignore stray semicolons
            TokenKind::Symbol(sym) if sym == ";" => {
                self.advance();
            }

            // --- Unknown: skip token gracefully instead of panicking
            _ => {
                println!("⚠️ Skipping unexpected token inside entity: {:?}", self.peek());
                self.advance();
            }
        }
    }

    self.expect(TokenKind::Symbol("}".into()), "Expected '}' to end entity");

    Some(Node::Entity(EntityNode { name, base, members }))
}



    fn parse_let(&mut self, is_const: bool) -> Option<Node> {
    // check if the next token is a type annotation like i32/i64/f64/i8/u64
    let mut explicit_type: Option<String> = None;

    // If next token looks like a type identifier (i32, f64, etc.)
    if let TokenKind::Identifier(type_name) = self.peek().clone() {
        if ["i8", "i32", "i64", "u64", "f64"].contains(&type_name.as_str()) {
            explicit_type = Some(type_name);
            self.advance(); // consume type identifier
        }
    }

    // expect variable name
    let var_name = match self.advance().clone() {
        TokenKind::Identifier(n) => n,
        _ => panic!("Expected variable name after let"),
    };

    // expect '='
    match self.advance().clone() {
        TokenKind::Symbol(s) if s == "=" => {}
        _ => panic!("Expected symbol '='"),
    }

    // parse expression (right-hand side)
    let expr = self.parse_expr();

    // optional ';'
    if let TokenKind::Symbol(semi) = self.peek() {
        if semi == ";" {
            self.advance();
        }
    }

    Some(Node::Let {
        name: var_name,
        value: expr,
        is_const,
        ty: explicit_type,

    })
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
    self.advance(); // consume 'let'
    if let Some(node) = self.parse_let(false) {
        init = Some(node);
    }
}
else {
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
fn parse_funcy(&mut self, is_async: bool) -> Expr {
    // expect function name
    let name = match self.advance().clone() {
        TokenKind::Identifier(n) => n,
        other => panic!("Expected function name after 'funcy', got {:?}", other),
    };

    // expect '('
    self.expect(TokenKind::Symbol("(".into()), "Expected '(' after function name");

    // === 🧠 parse parameters (with optional types)
    let mut params = Vec::new();

    if !self.check(TokenKind::Symbol(")".into())) {
        loop {
            // parameter name
            let param_name = match self.advance().clone() {
                TokenKind::Identifier(p) => p,
                other => panic!("Expected parameter name, got {:?}", other),
            };

            // optional type annotation like a: f32
            let mut param_type = "i32".to_string(); // default type
            if self.check(TokenKind::Symbol(":".into())) {
                self.advance(); // consume ':'
                match self.advance().clone() {
                    TokenKind::Identifier(ty_name) => {
                        println!("🧩 Parsed typed parameter: {}: {}", param_name, ty_name);
                        param_type = ty_name;
                    }
                    other => panic!("Expected type name after ':' in parameter, got {:?}", other),
                }
            }

            // ✅ Store full typed form ("a:f32")
            params.push(format!("{}:{}", param_name, param_type));

            // comma or end
            if self.check(TokenKind::Symbol(",".into())) {
                self.advance();
                continue;
            } else {
                break;
            }
        }
    }

    self.expect(TokenKind::Symbol(")".into()), "Expected ')' after parameters");

    // === 🏹 support arrow syntax: "=> expr"
    if self.check(TokenKind::Symbol("=".into())) {
        self.advance(); // consume '='
        if self.check(TokenKind::Symbol(">".into())) {
            self.advance(); // consume '>'

            // parse single-expression arrow body
            let expr = self.parse_expr();
            let body = vec![Node::Expr(Expr::Return(Some(Box::new(expr))))];

            return Expr::Funcy {
                name,
                params,
                body,
                is_async,
            };
        } else {
            panic!("Expected '>' after '=' for arrow function");
        }
    }

    // === 🧱 fallback to block-style: "{ ... }"
    self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start function body");

    let mut body = Vec::new();
    while !self.check(TokenKind::Symbol("}".into())) && !self.check(TokenKind::EOF) {
        if let Some(stmt) = self.parse_stmt() {
            body.push(stmt);
        } else {
            self.advance();
        }
    }

    self.expect(TokenKind::Symbol("}".into()), "Expected '}' to close function body");

    Expr::Funcy {
        name,
        params,
        body,
        is_async,
    }
}











}
impl Parser {
    /// Entry point for expression parsing
    pub fn parse_expr(&mut self) -> Expr {
    self.parse_assignment()
}

fn parse_assignment(&mut self) -> Expr {
    // 🧠 Start from logical OR, not equality
    let left = self.parse_logical_or();

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
    fn parse_logical_or(&mut self) -> Expr {
    let mut expr = self.parse_logical_and();

    while self.matches(&[TokenKind::Identifier("or".into())]) {
        let op = "or".to_string();
        let right = self.parse_logical_and();
        expr = Expr::BinaryOp {
            left: Box::new(expr),
            op,
            right: Box::new(right),
        };
    }

    expr
}

fn parse_logical_and(&mut self) -> Expr {
    let mut expr = self.parse_equality();

    while self.matches(&[TokenKind::Identifier("and".into())]) {
        let op = "and".to_string();
        let right = self.parse_equality();
        expr = Expr::BinaryOp {
            left: Box::new(expr),
            op,
            right: Box::new(right),
        };
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
    // ✅ handle 'await' keyword
    if self.check(TokenKind::Keyword("await".into())) {
        self.advance();
        let inner = self.parse_primary();
        return Expr::Await(Box::new(inner));
    }

    // ✅ handle array literals
    if self.check(TokenKind::Symbol("[".into())) {
        return self.parse_array_literal();
    }

    // ✅ handle object literals
    if self.check(TokenKind::Symbol("{".into())) {
        // But we must check if this is a *block* or an *object literal*.
        // If the next token is a string or identifier followed by ':', treat as object literal.
        if self.lookahead_is_object_literal() {
            return self.parse_object_literal();
        }
    }
    // ✅ Handle inline anonymous lambdas
if let TokenKind::Keyword(k) = self.peek() {
    if k == "func" || k == "funcy" {
        self.advance(); // consume 'func'

        // Optional name — if present, it's a normal function
        let mut name = String::new();
        if let TokenKind::Identifier(id) = self.peek().clone() {
            name = id.clone();
            self.advance();
        }

        self.expect(TokenKind::Symbol("(".into()), "Expected '(' after func");
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

        self.expect(TokenKind::Symbol(")".into()), "Expected ')' after params");

        // Parse the function body
        let body = self.parse_block();

        return Expr::Funcy {
            name,
            params,
            body,
            is_async: false,
        };
    }
}

    // ✅ fallback to existing literal/identifier logic
    match self.advance().clone() {
        TokenKind::Number { raw, ty } => Expr::TypedLiteral { value: raw, ty },
        TokenKind::String(s) => Expr::StringLiteral(s),
        TokenKind::Identifier(mut name) => {
    // 🔗 Merge dotted identifiers like "server.register" or "http.get"
    while self.check(TokenKind::Symbol(".".into())) {
        self.advance(); // consume '.'
        match self.advance().clone() {
            TokenKind::Identifier(next) => {
                name = format!("{}.{}", name, next);
            }
            other => panic!("Expected identifier after '.', got {:?}", other),
        }
    }

    // 📞 Handle function calls (after full name is built)
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

fn lookahead_is_object_literal(&self) -> bool {
    // Look ahead to detect `{ <identifier or string> :`
    let next = self.tokens.get(self.pos + 1);
    let next2 = self.tokens.get(self.pos + 2);

    matches!(next, Some(Token { kind: TokenKind::String(_), .. }) | Some(Token { kind: TokenKind::Identifier(_), .. }))
        && matches!(next2, Some(Token { kind: TokenKind::Symbol(s), .. }) if s == ":")
}

fn parse_array_literal(&mut self) -> Expr {
    self.expect(TokenKind::Symbol("[".into()), "Expected '[' to start array literal");
    let mut elements = Vec::new();

    if !self.check(TokenKind::Symbol("]".into())) {
        loop {
            elements.push(self.parse_expr());
            if self.check(TokenKind::Symbol("]".into())) {
                break;
            }
            self.expect(TokenKind::Symbol(",".into()), "Expected ',' between array elements");
        }
    }

    self.expect(TokenKind::Symbol("]".into()), "Expected ']' to close array literal");
    Expr::ArrayLiteral(elements)
}

fn parse_object_literal(&mut self) -> Expr {
    self.expect(TokenKind::Symbol("{".into()), "Expected '{' to start object literal");
    let mut fields = Vec::new();

    if !self.check(TokenKind::Symbol("}".into())) {
        loop {
            let key = match self.advance().clone() {
                TokenKind::String(s) => s,
                TokenKind::Identifier(id) => id,
                _ => panic!("Expected string or identifier as object key"),
            };
            self.expect(TokenKind::Symbol(":".into()), "Expected ':' after object key");
            let val = self.parse_expr();
            fields.push((key, val));

            if self.check(TokenKind::Symbol("}".into())) {
                break;
            }
            self.expect(TokenKind::Symbol(",".into()), "Expected ',' between object fields");
        }
    }

    self.expect(TokenKind::Symbol("}".into()), "Expected '}' to close object literal");
    Expr::ObjectLiteral(fields)
}

}





