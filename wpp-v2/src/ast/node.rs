use super::expr::Expr;

#[derive(Debug, Clone)]
pub enum Node {
    Let {
        name: String,
        value: Expr,
        is_const: bool, // ✅ new field
        ty: Option<String>,
    },
    Expr(Expr),
}

