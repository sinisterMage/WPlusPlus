use crate::ast::{Expr, Node};
use std::mem;

/// Simple W++ parser that turns text into AST nodes.
/// This can later be replaced with your real parser.
pub struct Parser {
    pub source: String,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }

    pub fn parse(&self) -> Vec<Node> {
        let mut nodes = Vec::new();

        let mut lines = self.source.lines().peekable();

while let Some(line) = lines.next() {
    let line = line.trim();
    if line.is_empty() || line == "{" || line == "}" {
        continue;
    }

    if line.starts_with("let ") {
        let parts: Vec<&str> = line.trim_end_matches(';').split('=').collect();
        if parts.len() == 2 {
            let left = parts[0].trim().strip_prefix("let").unwrap().trim();
            let right = parts[1].trim();
            let expr = self.parse_expr(right);
            nodes.push(Node::Let {
                name: left.to_string(),
                value: expr,
            });
        }
    } else if line.starts_with("if ") {
        let node = self.collect_if_block(line, &mut lines);
        nodes.push(node);
    }
    else if line.starts_with("while ") {
    let cond_start = line.find("while").unwrap() + 5;
    let cond_str = line[cond_start..].trim();

    let mut full_block = String::new();
    let mut brace_count = 0;
    let mut started = false;

    // Collect full block until matching '}'
    full_block.push_str(cond_str);
    for next_line in self.source.lines().skip_while(|l| !l.trim_start().starts_with("while ")) {
        let trimmed = next_line.trim();
        for ch in trimmed.chars() {
            if ch == '{' { brace_count += 1; started = true; }
            else if ch == '}' { brace_count -= 1; }
        }
        full_block.push(' ');
        full_block.push_str(trimmed);
        if started && brace_count == 0 { break; }
    }

    // Extract condition and body
    if let Some(idx) = full_block.find('{') {
        let (cond_raw, rest_raw) = full_block.split_at(idx);
        let cond_expr = self.parse_expr(cond_raw.trim());
        let body_str = rest_raw.trim_start_matches('{').trim_end_matches('}').trim();
        let body_nodes = self.parse_block(body_str);
        nodes.push(Node::Expr(Expr::While {
            cond: Box::new(cond_expr),
            body: body_nodes,
        }));
    }
}
else if line.starts_with("for ") {
    let mut full_block = String::new();
    full_block.push_str(line);

    let mut brace_count = line.chars().filter(|&c| c == '{').count()
        - line.chars().filter(|&c| c == '}').count();

    // ✅ Collect lines until matching closing brace
    while brace_count > 0 {
        if let Some(next_line) = lines.next() {
            full_block.push('\n');
            full_block.push_str(next_line);
            for ch in next_line.chars() {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }
        } else {
            break;
        }
    }

    // Now we have the *entire* for loop (header + body)
    let header_start = full_block.find("for").unwrap() + 3;
    let rest = full_block[header_start..].trim();

    // Split header and body
    let header_end = rest.find('{').expect("Missing '{' in for loop");
    let header = rest[..header_end].trim();
    let body_str = rest[header_end + 1..]
        .trim_end_matches('}')
        .trim();

    let parts: Vec<&str> = header.split(';').map(|s| s.trim()).collect();

    // --- Handle init ---
    let init = if !parts.is_empty() && !parts[0].is_empty() {
        let init_str = parts[0];
        if init_str.starts_with("let ") {
            let sub = Parser::new(init_str).parse();
            Some(Box::new(sub.into_iter().next().unwrap()))
        } else {
            Some(Box::new(Node::Expr(self.parse_expr(init_str))))
        }
    } else {
        None
    };

    // --- Handle cond ---
    let cond = if parts.len() > 1 && !parts[1].is_empty() {
        Some(Box::new(self.parse_expr(parts[1])))
    } else {
        None
    };

    // --- Handle post ---
    let post = if parts.len() > 2 && !parts[2].is_empty() {
        let post_str = parts[2];
        if post_str.starts_with("let ") {
            let sub = Parser::new(post_str).parse();
            Some(Box::new(sub.into_iter().next().unwrap()))
        } else {
            Some(Box::new(Node::Expr(self.parse_expr(post_str))))
        }
    } else {
        None
    };

    // --- Body ---
    let body = self.parse_block(body_str);

    nodes.push(Node::Expr(Expr::For {
        init,
        cond,
        post,
        body,
    }));
}




     else {
        nodes.push(Node::Expr(self.parse_expr(line)));
    }
}


        nodes
    }

    fn parse_expr(&self, expr: &str) -> Expr {
    let expr = expr.trim().trim_end_matches(';').trim();

    // ✅ Boolean literals
    if expr == "true" {
        return Expr::BoolLiteral(true);
    } else if expr == "false" {
        return Expr::BoolLiteral(false);
    }

    // ✅ String literal
    if expr.starts_with('"') && expr.ends_with('"') {
        return Expr::StringLiteral(expr[1..expr.len() - 1].to_string());
    }

    // ✅ Parentheses
    if expr.starts_with('(') && expr.ends_with(')') {
        return self.parse_expr(&expr[1..expr.len() - 1]);
    }

    // ✅ Assignment (=), but not ==, <=, >=
    if let Some(eq_idx) = expr.find('=') {
        let after = &expr[eq_idx..];
        if !after.starts_with("==") && !after.starts_with("<=") && !after.starts_with(">=") {
            let (l, r) = expr.split_at(eq_idx);
            return Expr::BinaryOp {
                left: Box::new(self.parse_expr(l.trim())),
                op: "=".to_string(),
                right: Box::new(self.parse_expr(r[1..].trim())),
            };
        }
    }

    // ✅ Comparison operators (<, >, <=, >=, ==, !=)
    for op in ["==", "!=", "<=", ">=", "<", ">"] {
        if let Some(idx) = expr.find(op) {
            let (l, r) = expr.split_at(idx);
            return Expr::BinaryOp {
                left: Box::new(self.parse_expr(l.trim())),
                op: op.to_string(),
                right: Box::new(self.parse_expr(r[op.len()..].trim())),
            };
        }
    }

    // ✅ Arithmetic (+, -, *, /)
    if expr.contains('+') || expr.contains('-') || expr.contains('*') || expr.contains('/') {
        return self.parse_chained_arithmetic(expr);
    }

    // ✅ Function call
    if let Some(idx) = expr.find('(') {
        let name = expr[..idx].trim().to_string();
        let args_str = expr[idx + 1..].trim_end_matches(')').trim();
        let args = if args_str.is_empty() {
            vec![]
        } else {
            args_str.split(',').map(|a| self.parse_expr(a.trim())).collect()
        };
        return Expr::Call { name, args };
    }

    // ✅ Literal or variable
    if let Ok(num) = expr.parse::<i32>() {
        Expr::Literal(num)
    } else {
        Expr::Variable(expr.to_string())
    }
}



fn parse_chained_arithmetic(&self, expr: &str) -> Expr {
    let tokens = Self::tokenize(expr);

    if tokens.is_empty() {
        panic!("Empty arithmetic expression");
    }

    fn precedence(op: &str) -> i32 {
        match op {
            "*" | "/" => 3,
            "+" | "-" => 2,
            _ => 0,
        }
    }

    let mut output: Vec<Expr> = Vec::new();
    let mut ops: Vec<String> = Vec::new();

    let mut prev_token_was_op = true; // For unary minus detection

    for token in tokens {
        match token.as_str() {
            "(" => {
                ops.push(token);
                prev_token_was_op = true;
            }
            ")" => {
                while let Some(top) = ops.pop() {
                    if top == "(" {
                        break;
                    }
                    let right = Box::new(output.pop().expect("Missing RHS"));
                    let left = Box::new(output.pop().expect("Missing LHS"));
                    output.push(Expr::BinaryOp { left, op: top, right });
                }
                prev_token_was_op = false;
            }
            "+" | "-" | "*" | "/" => {
                // Detect unary minus
                if token == "-" && prev_token_was_op {
                    // Unary minus → treat as (0 - expr)
                    ops.push("u-".to_string());
                    prev_token_was_op = true;
                    continue;
                }

                while let Some(top) = ops.last() {
                    if top != "(" && precedence(top) >= precedence(&token) {
                        let right = Box::new(output.pop().expect("Missing RHS"));
                        let left = Box::new(output.pop().expect("Missing LHS"));
                        let op = ops.pop().unwrap();
                        output.push(Expr::BinaryOp { left, op, right });
                    } else {
                        break;
                    }
                }
                ops.push(token);
                prev_token_was_op = true;
            }
            _ => {
                // number or variable
                let expr_node = if let Ok(num) = token.parse::<i32>() {
                    Expr::Literal(num)
                } else {
                    Expr::Variable(token)
                };

                // Check if unary minus is pending
                if let Some(last_op) = ops.last() {
                    if last_op == "u-" {
                        ops.pop();
                        let left = Box::new(Expr::Literal(0));
                        let right = Box::new(expr_node);
                        output.push(Expr::BinaryOp {
                            left,
                            op: "-".to_string(),
                            right,
                        });
                        prev_token_was_op = false;
                        continue;
                    }
                }

                output.push(expr_node);
                prev_token_was_op = false;
            }
        }
    }

    // Final unwinding
    while let Some(op) = ops.pop() {
        if op == "(" {
            panic!("Unmatched '(' in expression: {}", expr);
        }

        let right = Box::new(output.pop().expect("Missing RHS"));
        let left = Box::new(output.pop().expect("Missing LHS"));
        output.push(Expr::BinaryOp { left, op, right });
    }

    output.pop().unwrap()
}

fn tokenize(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for (i, ch) in expr.chars().enumerate() {
    match ch {
        ' ' => {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        }

        '+' | '-' | '*' | '/' | '(' | ')' => {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }

            // ✅ Special case:
            // Only treat '(' as a *token* if it is NOT part of a function call (like print(...))
            if ch == '(' {
                // Peek previous non-space character (if any)
                if i > 0 {
                    let prev = expr[..i].chars().rev().find(|c| !c.is_whitespace());
                    if let Some(p) = prev {
                        if p.is_alphabetic() {
                            // previous char is part of an identifier, e.g. "print("
                            // → skip pushing "(" here; let function call parser handle it
                            continue;
                        }
                    }
                }
            }

            tokens.push(ch.to_string());
        }

        _ => current.push(ch),
    }
}

// Push any trailing token
if !current.is_empty() {
    tokens.push(current.clone());
    current.clear();
}



    tokens
}




}
impl Parser {
    fn parse_block(&self, block: &str) -> Vec<Node> {
    block
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && *l != "{" && *l != "}" && !l.starts_with("else"))
        .map(|l| {
            if l.starts_with("let ") {
                let parts: Vec<&str> = l.trim_end_matches(';').split('=').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().strip_prefix("let").unwrap().trim().to_string();
                    let value = self.parse_expr(parts[1].trim());
                    Node::Let { name, value }
                } else {
                    panic!("Invalid let syntax: {}", l);
                }
            } else if l.starts_with("if ") {
                // recursively handle nested if
                let nested = Parser { source: l.to_string() }.parse();
                nested.into_iter().next().unwrap()
            } else {
                Node::Expr(self.parse_expr(l))
            }
        })
        .collect()
}

}
impl Parser {
    /// Collect a complete `if ... { ... } else { ... }` block, even across multiple lines.
    fn collect_if_block<'a>(
    &self,
    first_line: &str,
    lines: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Node {
    // Condition is on the same line before the first '{'
    let mut cond_str = String::new();
    let mut then_body = String::new();
    let mut else_body = String::new();

    let mut brace_count = 0;
    let mut in_then = false;
    let mut in_else = false;

    // Split first line into "if <cond>" and maybe start of block
    if let Some(idx) = first_line.find('{') {
        cond_str = first_line[..idx].trim_start_matches("if").trim().to_string();
        brace_count = 1;
        in_then = true;
    } else {
        panic!("Expected '{{' after if condition: {}", first_line);
    }

    // Read until braces close
    while let Some(next) = lines.next() {
        let line = next.trim();

        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
                if in_else {
                    // inside else block
                }
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    if in_then {
                        in_then = false;
                    } else if in_else {
                        in_else = false;
                    }
                    if !in_then && !in_else {
                        break;
                    }
                }
            }
        }

        if in_then {
            if line.starts_with("else") {
                in_then = false;
                in_else = true;
            } else {
                then_body.push_str(line);
                then_body.push('\n');
            }
        } else if in_else {
            else_body.push_str(line);
            else_body.push('\n');
        }

        if brace_count == 0 {
            break;
        }
    }

    // Parse condition and blocks
    let cond_expr = self.parse_expr(&cond_str);
    let then_nodes = self.parse_block(&then_body);
    let else_nodes = if else_body.is_empty() {
        None
    } else {
        Some(self.parse_block(&else_body))
    };

    Node::Expr(Expr::If {
        cond: Box::new(cond_expr),
        then_branch: then_nodes,
        else_branch: else_nodes,
    })
}


    /// Actually parse the collected `if {..} else {..}` into AST
    fn parse_if_block(&self, block: &str) -> Node {
        // Example input:
        // if x < 5 { print("hi") } else { print("bye") }

        let mut else_part = None;

        // Find the first '{'
        if let Some(open_idx) = block.find('{') {
            let (cond_raw, after_cond) = block.split_at(open_idx);
            let cond_str = cond_raw.trim_start_matches("if").trim();

            // Find matching '}' for the first block
            if let Some(close_idx) = after_cond.find('}') {
                let then_body = &after_cond[1..close_idx]; // skip the first '{'
                let after_then = after_cond[close_idx + 1..].trim();

                // Handle optional else
                if after_then.starts_with("else") {
                    if let Some(open2) = after_then.find('{') {
                        if let Some(close2) = after_then.rfind('}') {
                            let else_body = &after_then[open2 + 1..close2];
                            else_part = Some(self.parse_block(else_body));
                        }
                    }
                }

                // Parse condition and bodies
                let cond_expr = self.parse_expr(cond_str);
                let then_nodes = self.parse_block(then_body);

                return Node::Expr(Expr::If {
                    cond: Box::new(cond_expr),
                    then_branch: then_nodes,
                    else_branch: else_part,
                });
            }
        }

        panic!("Invalid if syntax: {}", block);
    }
}
