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
    } else {
        nodes.push(Node::Expr(self.parse_expr(line)));
    }
}


        nodes
    }

    fn parse_expr(&self, expr: &str) -> Expr {
    let expr = expr.trim().trim_end_matches(';').trim();

    // ✅ Booleans
    if expr == "true" {
        return Expr::BoolLiteral(true);
    } else if expr == "false" {
        return Expr::BoolLiteral(false);
    }

    // ✅ String literal
    if expr.starts_with('"') && expr.ends_with('"') {
        return Expr::StringLiteral(expr[1..expr.len()-1].to_string());
    }

    // ✅ Function call
    if let Some(idx) = expr.find('(') {
        let name = expr[..idx].trim().to_string();
        let args_str = expr[idx + 1..]
            .trim_end_matches(')')
            .trim_end_matches(';')
            .trim();
        let args = if args_str.is_empty() {
            vec![]
        } else {
            args_str
                .split(',')
                .map(|a| self.parse_expr(a.trim()))
                .collect()
        };
        return Expr::Call { name, args };
    }

    // ✅ Comparison operators
    let cmp_ops = ["==", "!=", "<=", ">=", "<", ">"];
    for op in cmp_ops {
        if let Some(idx) = expr.find(op) {
            let (l, r) = expr.split_at(idx);
            let left = self.parse_expr(l.trim());
            let right = self.parse_expr(r[op.len()..].trim());
            return Expr::BinaryOp {
                left: Box::new(left),
                op: op.to_string(),
                right: Box::new(right),
            };
        }
    }

    // ✅ Arithmetic
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.len() == 1 {
        if let Ok(num) = tokens[0].parse::<i32>() {
            Expr::Literal(num)
        } else {
            Expr::Variable(tokens[0].to_string())
        }
    } else if tokens.len() == 3 {
        let left = self.parse_expr(tokens[0]);
        let right = self.parse_expr(tokens[2]);
        Expr::BinaryOp {
            left: Box::new(left),
            op: tokens[1].to_string(),
            right: Box::new(right),
        }
    } else {
        panic!("Unsupported expression: {}", expr);
    }
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
