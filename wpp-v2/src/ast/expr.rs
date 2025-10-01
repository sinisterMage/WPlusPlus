use super::node::Node; // 👈 to use Node inside Expr

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    Variable(String),
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Vec<Node>,
        else_branch: Option<Vec<Node>>,
    },
    While {
        cond: Box<Expr>,
        body: Vec<Node>,
    },
    For {
        init: Option<Box<Node>>,
        cond: Option<Box<Expr>>,
        post: Option<Box<Expr>>,
        body: Vec<Node>,
    },
    Break,
    Continue,
    Switch {
    expr: Box<Expr>,
    cases: Vec<(Expr, Vec<Node>)>, // Each case: condition + block
    default: Option<Vec<Node>>,    // Optional default block
},
    TryCatch {
        try_block: Vec<Node>,
        catch_var: Option<String>,
        catch_block: Vec<Node>,
        finally_block: Option<Vec<Node>>, // ✅ NEW
    },
    Throw {
        expr: Box<Expr>,
    },

}

