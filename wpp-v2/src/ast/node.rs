use super::expr::Expr;

#[derive(Debug, Clone)]
pub enum Node {
    Let { name: String, value: Expr },
    Expr(Expr),
}
